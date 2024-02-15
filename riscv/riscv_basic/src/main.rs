//! examples/lab4_example

#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use core::panic::PanicInfo;
use syncrim_clic_rt as _;
use riscv_rt as _;
//static mut COUNTER:u32 = 0x1337;

#[rtic::app(device = clic)]
mod app{
    use embedded_hal::digital::StatefulOutputPin;
    use syncrim_hal::gpio::Pins;
    #[shared]
    struct Shared{}

    #[local]
    struct Local{
        pended: bool,
    }
    
    #[init]
    fn init(_:init::Context) -> (Shared, Local) {
        let pended = false;
        rtic::export::pend(clic::Interrupt2);
        (Shared{},Local{pended})
    }

    #[idle]
    fn idle(_:idle::Context) ->! {
        let peripherals = syncrim_pac::Peripherals::take().unwrap();
        let gpio = peripherals.GPIO;
        let pins = Pins::new(gpio);
        let mut pin = pins.pin2.into_output();
        loop{
            for _ in 0..10_000{
                riscv::asm::nop()
            }
            pin.toggle().ok();
        }
    }
    #[task(binds = Interrupt1, priority = 1)]
    fn i1(_:i1::Context){
    }
    #[task(binds = Interrupt2, priority = 2)]
    fn i2(_:i2::Context){
        rtic::export::pend(clic::Interrupt1);
        rtic::export::pend(clic::Interrupt4);
    }
    #[task(binds = Interrupt3, priority = 3)]
    fn i3(_:i3::Context){
    }
    #[task(binds = Interrupt4, priority = 4, local=[pended])]
    fn i4(cx:i4::Context){
        if !*cx.local.pended{
            *cx.local.pended = true;
            rtic::export::pend(clic::Interrupt6);
        }
    }
    #[task(binds = Interrupt5, priority = 5)]
    fn i5(_:i5::Context){
    }
    #[task(binds = Interrupt6, priority = 6)]
    fn i6(_:i6::Context){
        rtic::export::pend(clic::Interrupt8);
        rtic::export::pend(clic::Interrupt3);
    }
    #[task(binds = Interrupt7, priority = 7)]
    fn i7(_:i7::Context){
    }
    #[task(binds = Interrupt8, priority = 8)]
    fn i8(_:i8::Context){
        rtic::export::pend(clic::Interrupt4);
        rtic::export::pend(clic::Interrupt5);
    }

}
#[panic_handler]
unsafe fn _panic(_:&PanicInfo)->!{
    loop{}
}
