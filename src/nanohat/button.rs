use futures::stream::StreamExt;
use gpio_cdev::{AsyncLineEventHandle, Chip, Error, EventRequestFlags, LineRequestFlags};
use std::path::Path;

pub struct Button {
    f1_handle: AsyncLineEventHandle,
    f2_handle: AsyncLineEventHandle,
    f3_handle: AsyncLineEventHandle,
}
impl Button {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let mut chip = Chip::new(path)?;

        let offsets = [0, 2, 3];
        let lines = chip.get_lines(&offsets)?;

        let event_handles = (0..offsets.len())
            .map(|i| {
                lines[i].events(
                    LineRequestFlags::INPUT,
                    EventRequestFlags::RISING_EDGE,
                    "PressEvent",
                )
            })
            .collect::<Result<Vec<_>, Error>>()?;

        let mut async_event_handles = event_handles
            .into_iter()
            .map(|handle| AsyncLineEventHandle::new(handle))
            .collect::<Result<Vec<_>, Error>>()?;

        assert_eq!(async_event_handles.len(), 3);

        Ok(Self {
            f3_handle: async_event_handles.pop().unwrap(),
            f2_handle: async_event_handles.pop().unwrap(),
            f1_handle: async_event_handles.pop().unwrap(),
        })
    }
    pub async fn pressed(&mut self) -> Result<[bool; 3], Error> {
        let pressed_button = tokio::select! {
            _ = self.f1_handle.next() => [true, false, false],
            _ = self.f2_handle.next() => [false, true, false],
            _ = self.f3_handle.next() => [false, false, true],
        };
        Ok([
            (self.f1_handle.as_ref().get_value()? == 1) | pressed_button[0],
            (self.f2_handle.as_ref().get_value()? == 1) | pressed_button[1],
            (self.f3_handle.as_ref().get_value()? == 1) | pressed_button[2],
        ])
    }
}

#[cfg(target_os = "linux")]
#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(test)]
    fn open_test() {
        let button = Button::open("");
        assert!(Button::open("").is_err());
        assert!(Button::open("/dev/gpiochip0").is_ok());
    }
}
