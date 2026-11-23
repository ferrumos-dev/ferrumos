# Why FerrumOS uses the MIT License

FerrumOS is a teaching and research kernel. We want a student to be able to take one driver into a course project and a hardware company to be able to adapt the same driver for a device it sells. MIT permits both uses under one condition that either developer can meet by including a copy of the license. It also carries forward the license already named by the [reference prototype](../../../../POC/ferrumos_PoC/README.md#license).

| Intended use | What MIT allows | What the user must do |
| --- | --- | --- |
| Copy the kernel, run it or study it | Make and use copies free of charge. | Keep the copyright and permission notices with copies or substantial portions. |
| Modify a driver in a private fork | Change, combine and test the code, and keep the changes private. | Retain the notices in the copy. Sending the changes upstream is voluntary. |
| Publish or sell a modified fork | Distribute the source or binaries, charge a fee and sublicense the software. | Include the original notices with what recipients receive. The simplest way is to ship the full `LICENSE`. |
| Ship FerrumOS code in a proprietary device | Sell the device, distribute a binary and keep the vendor's source and modifications private. MIT requires no royalties or upstream contribution. | Include the FerrumOS copyright and permission notices with the device, for example in the third-party notices supplied with its firmware. |

A vendor that removes the FerrumOS notices and ships a substantial part of the framebuffer driver breaches the license condition, even if it rewrote other parts of the driver. The same condition applies to a fork shared for free.

The trade-off for the team is that a company can improve the driver and sell the result without publishing its changes. We accept that in exchange for making individual components easy to reuse. MIT also supplies the software without warranty and limits the authors’ liability.

The grant covers the project’s own software and associated documentation. Dependencies such as `bootloader` and `pc-keyboard`, and material credited to other authors, retain their own terms.

**References** · [FerrumOS license](./LICENSE) · [MIT text at the Open Source Initiative](https://opensource.org/license/mit) · [Permissions and conditions at Choose a License](https://choosealicense.com/licenses/mit/)
