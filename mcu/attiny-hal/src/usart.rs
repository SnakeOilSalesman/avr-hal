#[allow(unused_imports)]
use crate::port;
pub use avr_hal_generic::usart::*;

pub type TinyUsart<USART, RX, TX, CLOCK> =
    avr_hal_generic::usart::Usart<crate::Attiny, USART, RX, TX, CLOCK>;
pub type TinyUsartWriter<USART, RX, TX, CLOCK> =
    avr_hal_generic::usart::UsartWriter<crate::Attiny, USART, RX, TX, CLOCK>;
pub type TinyUsartReader<USART, RX, TX, CLOCK> =
    avr_hal_generic::usart::UsartReader<crate::Attiny, USART, RX, TX, CLOCK>;

#[cfg(any(feature = "attiny828"))]
pub type Usart<CLOCK> = TinyUsart<
    crate::pac::USART,
    port::Pin<port::mode::Input, port::PC2>,
    port::Pin<port::mode::Output, port::PC3>,
    CLOCK,
>;


// ATtiny828 has unnumbered single USART module
#[cfg(any(feature = "attiny828"))]
impl
    crate::usart::UsartOps<
        crate::Attiny,
        crate::port::Pin<crate::port::mode::Input, port::PC2>,
        crate::port::Pin<crate::port::mode::Output, port::PC3>,
    > for crate::pac::USART
{
    fn raw_init<CLOCK>(&mut self, baudrate: crate::usart::Baudrate<CLOCK>) {
        self.ubrr().write(|w| w.set(baudrate.ubrr));
        self.ucsra().write(|w| w.u2x().bit(baudrate.u2x));

        // Enable receiver and transmitter but leave interrupts disabled.
        self.ucsrb()
            .write(|w| w.txen().set_bit().rxen().set_bit());

        // Set frame format to 8n1 for now.  At some point, this should be made
        // configurable, similar to what is done in other HALs.
        #[rustfmt::skip]
        self.ucsrc().write(|w| w
            .umsel().usart_async()
            .ucsz().chr8()
            .usbs().stop1()
            .upm().disabled()
        );
    }

    fn raw_deinit(&mut self) {
        // Wait for any ongoing transfer to finish.
        avr_hal_generic::nb::block!(self.raw_flush()).ok();
        self.ucsrb().reset();
    }

    fn raw_flush(&mut self) -> avr_hal_generic::nb::Result<(), core::convert::Infallible> {
        if self.ucsra().read().udre().bit_is_clear() {
            Err(avr_hal_generic::nb::Error::WouldBlock)
        } else {
            Ok(())
        }
    }

    fn raw_write(
        &mut self,
        byte: u8,
    ) -> avr_hal_generic::nb::Result<(), core::convert::Infallible> {
        // Call flush to make sure the data-register is empty
        self.raw_flush()?;

        self.udr().write(|w| w.set(byte));
        Ok(())
    }

    fn raw_read(&mut self) -> avr_hal_generic::nb::Result<u8, core::convert::Infallible> {
        if self.ucsra().read().rxc().bit_is_clear() {
            return Err(avr_hal_generic::nb::Error::WouldBlock);
        }

        Ok(self.udr().read().bits())
    }

    fn raw_interrupt(&mut self, event: crate::usart::Event, state: bool) {
        match event {
            crate::usart::Event::RxComplete => {
                self.ucsrb().modify(|_, w| w.rxcie().bit(state));
            }
            crate::usart::Event::TxComplete => {
                self.ucsrb().modify(|_, w| w.txcie().bit(state));
            }
            crate::usart::Event::DataRegisterEmpty => {
                self.ucsrb().modify(|_, w| w.udrie().bit(state));
            }
        }
    }
}
