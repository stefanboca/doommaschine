use image::{imageops::ColorMap, Rgba};

#[derive(Clone, Copy)]
pub struct BiLevelRgba;

impl ColorMap for BiLevelRgba {
    type Color = Rgba<u8>;

    #[inline(always)]
    fn index_of(&self, color: &Rgba<u8>) -> usize {
        if color.0[0] > 127 || color.0[1] > 127 || color.0[2] > 127 {
            1
        } else {
            0
        }
    }

    #[inline(always)]
    fn lookup(&self, idx: usize) -> Option<Self::Color> {
        match idx {
            0 => Some([0x00, 0x00, 0x00, 0xFF].into()),
            1 => Some([0xFF, 0xFF, 0xFF, 0xFF].into()),
            _ => None,
        }
    }

    /// Indicate NeuQuant implements `lookup`.
    fn has_lookup(&self) -> bool {
        true
    }

    #[inline(always)]
    fn map_color(&self, color: &mut Rgba<u8>) {
        color.0 = if self.index_of(color) == 0 {
            [0x00, 0x00, 0x00, 0xFF]
        } else {
            [0xFF, 0xFF, 0xFF, 0xFF]
        };
    }
}
