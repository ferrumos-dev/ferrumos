
use spin::Mutex;


pub const LEVELS: usize = 3;
pub const MAX_TASKS_PER_QUEUE: usize = 16;
pub const LEVEL_QUANTA: [u32; LEVELS] = [2, 4, 8];
pub const AGING_THRESHOLD: u32 = 60;


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskType {
    Interactive,
    CpuBound,
    IoBound,
}

impl TaskType {
    pub fn name(&self) -> &'static str {
        match self {
            TaskType::Interactive => "interactive",
            TaskType::CpuBound   => "cpu-bound ",
            TaskType::IoBound    => "io-bound  ",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskState { Ready, Running, Finished }


#[derive(Debug, Clone, Copy)]
pub struct Task {
    pub id:           u32,
    pub name:         [u8; 16],
    pub name_len:     usize,
    pub priority:     usize,
    pub quantum_used: u32,
    pub total_ticks:  u32,
    pub wait_ticks:   u32,
    pub task_type:    TaskType,
    pub state:        TaskState,
    pub work_left:    u32,
    pub io_counter:   u32,
}

impl Task {
    pub fn new(id: u32, name: &str, task_type: TaskType, work_ticks: u32) -> Self {
        let mut name_buf = [0u8; 16];
        let bytes = name.as_bytes();
        let len = bytes.len().min(15);
        name_buf[..len].copy_from_slice(&bytes[..len]);
        Task {
            id, name: name_buf, name_len: len,
            priority: 0, quantum_used: 0, total_ticks: 0, wait_ticks: 0,
            task_type, state: TaskState::Ready, work_left: work_ticks, io_counter: 0,
        }
    }

    pub fn name_str(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("?")
    }

    fn simulate_tick(&mut self) -> bool {
        self.quantum_used += 1;
        self.total_ticks  += 1;
        self.work_left     = self.work_left.saturating_sub(1);
        self.io_counter   += 1;
        match self.task_type {
            TaskType::Interactive => true,
            TaskType::CpuBound   => false,
            TaskType::IoBound    => self.io_counter % 3 == 0,
        }
    }
}


pub struct FixedQueue {
    tasks: [Option<Task>; MAX_TASKS_PER_QUEUE],
    head:  usize,
    len:   usize,
}

impl FixedQueue {
    pub const fn new() -> Self {
        FixedQueue { tasks: [None; MAX_TASKS_PER_QUEUE], head: 0, len: 0 }
    }
    pub fn is_empty(&self) -> bool { self.len == 0 }
    pub fn len(&self)      -> usize { self.len }

    pub fn push(&mut self, task: Task) -> bool {
        if self.len >= MAX_TASKS_PER_QUEUE { return false; }
        let tail = (self.head + self.len) % MAX_TASKS_PER_QUEUE;
        self.tasks[tail] = Some(task);
        self.len += 1;
        true
    }

    pub fn pop(&mut self) -> Option<Task> {
        if self.len == 0 { return None; }
        let t = self.tasks[self.head].take();
        self.head = (self.head + 1) % MAX_TASKS_PER_QUEUE;
        self.len -= 1;
        t
    }

    pub fn iter(&self) -> FixedQueueIter<'_> {
        FixedQueueIter { queue: self, pos: 0 }
    }

    pub fn drain_aged(&mut self, threshold: u32, aged_out: &mut FixedQueue) {
        let count = self.len;
        for _ in 0..count {
            if let Some(mut t) = self.pop() {
                if t.wait_ticks >= threshold {
                    t.wait_ticks = 0; t.quantum_used = 0;
                    let _ = aged_out.push(t);
                } else {
                    let _ = self.push(t);
                }
            }
        }
    }
}

pub struct FixedQueueIter<'a> {
    queue: &'a FixedQueue,
    pos:   usize,
}
impl<'a> Iterator for FixedQueueIter<'a> {
    type Item = &'a Task;
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.queue.len { return None; }
        let idx = (self.queue.head + self.pos) % MAX_TASKS_PER_QUEUE;
        self.pos += 1;
        self.queue.tasks[idx].as_ref()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Metrics {
    pub total_ticks:      u32,
    pub total_scheduled:  u32,
    pub total_demotions:  u32,
    pub total_promotions: u32,
    pub total_aged:       u32,
    pub completed_tasks:  u32,
    pub sum_wait_ticks:   u64,
    pub max_wait_ticks:   u32,
}

impl Metrics {
    pub const fn new() -> Self {
        Metrics {
            total_ticks: 0, total_scheduled: 0, total_demotions: 0,
            total_promotions: 0, total_aged: 0, completed_tasks: 0,
            sum_wait_ticks: 0, max_wait_ticks: 0,
        }
    }
    pub fn avg_wait(&self) -> u32 {
        if self.completed_tasks == 0 { 0 }
        else { (self.sum_wait_ticks / self.completed_tasks as u64) as u32 }
    }
    pub fn throughput_x100(&self) -> u32 {
        if self.total_ticks == 0 { 0 }
        else { self.completed_tasks * 10000 / self.total_ticks }
    }
}


pub struct Scheduler {
    pub queues:  [FixedQueue; LEVELS],
    pub running: Option<Task>,
    pub metrics: Metrics,
    pub next_id: u32,
}

impl Scheduler {
    pub const fn new() -> Self {
        Scheduler {
            queues:  [FixedQueue::new(), FixedQueue::new(), FixedQueue::new()],
            running: None,
            metrics: Metrics::new(),
            next_id: 1,
        }
    }

    pub fn spawn(&mut self, name: &str, task_type: TaskType, work_ticks: u32) {
        let id = self.next_id;
        self.next_id += 1;
        let _ = self.queues[0].push(Task::new(id, name, task_type, work_ticks));
    }

    pub fn tick(&mut self) {
        self.metrics.total_ticks += 1;

        for level in 0..LEVELS {
            let count = self.queues[level].len();
            for _ in 0..count {
                if let Some(mut t) = self.queues[level].pop() {
                    t.wait_ticks += 1;
                    let _ = self.queues[level].push(t);
                }
            }
        }
        if let Some(mut task) = self.running.take() {
            let early_yield = task.simulate_tick();
            if task.work_left == 0 {
                self.metrics.completed_tasks += 1;
                self.metrics.sum_wait_ticks  += task.wait_ticks as u64;
                if task.wait_ticks > self.metrics.max_wait_ticks {
                    self.metrics.max_wait_ticks = task.wait_ticks;
                }
            } else if early_yield {
                let nl = task.priority.saturating_sub(1);
                if nl < task.priority { self.metrics.total_promotions += 1; }
                task.priority = nl; task.quantum_used = 0;
                task.state = TaskState::Ready;
                let _ = self.queues[nl].push(task);
            } else if task.quantum_used >= LEVEL_QUANTA[task.priority] {
                let nl = (task.priority + 1).min(LEVELS - 1);
                if nl > task.priority { self.metrics.total_demotions += 1; }
                task.priority = nl; task.quantum_used = 0;
                task.state = TaskState::Ready;
                let _ = self.queues[nl].push(task);
            } else {
                self.running = Some(task);
                return;
            }
        }

        for level in 1..LEVELS {
            let mut promoted = FixedQueue::new();
            self.queues[level].drain_aged(AGING_THRESHOLD, &mut promoted);
            while let Some(mut t) = promoted.pop() {
                t.priority = level.saturating_sub(1);
                let _ = self.queues[t.priority].push(t);
                self.metrics.total_aged += 1;
            }
        }

        for level in 0..LEVELS {
            if !self.queues[level].is_empty() {
                if let Some(mut next) = self.queues[level].pop() {
                    next.state = TaskState::Running;
                    next.wait_ticks = 0;
                    self.metrics.total_scheduled += 1;
                    self.running = Some(next);
                    return;
                }
            }
        }
    }

    pub fn task_count(&self) -> usize {
        self.queues.iter().map(|q| q.len()).sum::<usize>()
            + if self.running.is_some() { 1 } else { 0 }
    }
    pub fn queue_len(&self, level: usize) -> usize { self.queues[level].len() }
}


pub static SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());

pub fn tick() {
    if let Some(mut sched) = SCHEDULER.try_lock() {
        sched.tick();
    }
}

pub fn spawn(name: &str, task_type: TaskType, work_ticks: u32) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        SCHEDULER.lock().spawn(name, task_type, work_ticks);
    });
}

pub fn snapshot() -> (u32, usize, alloc::string::String, usize, usize, usize, u32, u32, u32) {
    use alloc::string::String;

    x86_64::instructions::interrupts::without_interrupts(|| {
        let sched = SCHEDULER.lock();
        let running_name = if let Some(ref task) = sched.running {
            String::from(task.name_str())
        } else {
            String::from("idle")
        };

        (
            sched.metrics.total_ticks,
            sched.task_count(),
            running_name,
            sched.queue_len(0),
            sched.queue_len(1),
            sched.queue_len(2),
            sched.metrics.completed_tasks,
            sched.metrics.avg_wait(),
            sched.metrics.throughput_x100(),
        )
    })
}
