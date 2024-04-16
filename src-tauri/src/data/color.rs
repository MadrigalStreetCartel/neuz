#[derive(Debug, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub tolerance: Option<u8>,
}

impl Color {
    pub fn new(color: [u8; 3], tolerance: Option<u8>) -> Self {
        Self {
            r: color[0],
            g: color[1],
            b: color[2],
            tolerance,
        }
    }
}

impl Clone for Color {
    fn clone(&self) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            tolerance: self.tolerance,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ColorDetection {
    pub colors: Vec<Color>,
    pub tolerance: u8,
}

impl ColorDetection {
    pub fn from(color: Vec<[u8; 3]>) -> Self{
        Self {
            colors: {
                let mut colors = vec![];
                for c in color {
                    colors.push(Color::new(c, None));
                }
                colors
            },
            tolerance: 5,
        }
    }

/*     pub fn from_with_tolerance(color: Vec<[u8; 4]>, tolerance: u8) -> Self{
        Self {
            colors: {
                let mut colors = vec![];
                for c in color {
                    colors.push(Color::new([c[0],c[1],c[2]], Some(c[3])));
                }
                colors
            },
            tolerance,
        }
    } */
    #[inline(always)]
    pub fn color_match(&self, color: &[u8; 4]) -> bool {
        // Color matching based on tolerance
        for ref_color in &self.colors {
            let r_diff = ((color[0] as i16) - (ref_color.r as i16)).abs();
            if r_diff > (ref_color.tolerance.unwrap_or(self.tolerance as u8) as i16) {
                continue;
            }
            let g_diff = ((color[1] as i16) - (ref_color.g as i16)).abs();
            if g_diff > (ref_color.tolerance.unwrap_or(self.tolerance as u8) as i16) {
                continue;
            }
            let b_diff = ((color[2] as i16) - (ref_color.b as i16)).abs();
            if b_diff > (ref_color.tolerance.unwrap_or(self.tolerance as u8) as i16) {
                continue;
            }
            return true;
        }
        false
    }
}

impl Clone for ColorDetection {
    fn clone(&self) -> Self {
        Self {
            colors: self.colors.clone(),
            tolerance: self.tolerance,
        }
    }
}

impl Default for ColorDetection {
    fn default() -> Self {
        Self {
            colors: vec![],
            tolerance: 0,
        }
    }
}
