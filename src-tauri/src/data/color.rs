use std::ops::Sub;

use palette::{ rgb::{ self, Rgb }, FromColor, Hsv, IntoColor, Srgb };

#[derive(Debug, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub rgb_tolerance: Option<u8>,
    pub hsv_tolerance: Option<[f32; 3]>,
    pub hsv: Option<Hsv>,
}

impl Color {
    fn new_rgb(rgb: [u8; 3], rgb_tolerance: Option<u8>) -> Self {
        Self {
            r: rgb[0],
            g: rgb[1],
            b: rgb[2],
            rgb_tolerance,
            hsv: None,
            hsv_tolerance: None,
        }
    }

    // Constructeur pour les couleurs HSL qui convertit en RGB
    fn new_hsv(hsv: [f32; 3], hsv_tolerance: Option<[f32; 3]>) -> Self {
        let hsv = Hsv::new(hsv[0], hsv[1], hsv[2]);
        let rgb = Srgb::from_color(hsv);
        let rgb = rgb.into_format();
        Self {
            r: rgb.red,
            g: rgb.green,
            b: rgb.blue,
            rgb_tolerance: None,
            hsv: Some(hsv),
            hsv_tolerance,
        }
    }

    fn to_hsv(pixel: [u8; 4]) -> Hsv {
        Hsv::from_color(Srgb::new(pixel[0], pixel[1], pixel[2]).into_format())
    }

    pub fn match_pixel(&self, pixel: [u8; 4], label: String) -> bool {
        if let Some(hsv) = self.hsv {
            let color_hsv = Self::to_hsv(pixel);
            let tolerance = self.hsv_tolerance.unwrap_or([0.0, 0.0, 0.0]);
            let hsv_diff = color_hsv.sub(hsv);
            let hue_diff = color_hsv.hue.into_positive_degrees() - hsv.hue.into_positive_degrees();

            if
                hue_diff.abs() <= tolerance[0] &&
                hsv_diff.saturation.abs() <= tolerance[1] &&
                hsv_diff.value.abs() <= tolerance[2]
            {
               /*  if label == "TargetAggressive" { // uSE FULL FOR DEBUG
                    //if color_hsv.hue.into_positive_degrees() > 238.0 && color_hsv.hue.into_positive_degrees() < 121.0 {
                        println!("Hsv diff: hue {}, sat {}, val {} color: {:?}", hue_diff.abs(), hsv_diff.saturation.abs(), hsv_diff.value.abs(), color_hsv);
                        //println!("Compared color: {:?} and {:?} with tolerance: {:?}", color_hsv, hsv, tolerance);
                   // }
                } */
                return true;
            } else {
                return false;
            }
        } else {
            if
                ((self.r as i32) - (pixel[0] as i32)).abs() <=
                    (self.rgb_tolerance.unwrap_or(0) as i32) &&
                ((self.g as i32) - (pixel[1] as i32)).abs() <=
                    (self.rgb_tolerance.unwrap_or(0) as i32) &&
                ((self.b as i32) - (pixel[2] as i32)).abs() <=
                    (self.rgb_tolerance.unwrap_or(0) as i32)
            {
                return true;
            }
        }
        false
    }
}

impl Clone for Color {
    fn clone(&self) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            rgb_tolerance: self.rgb_tolerance,
            hsv: self.hsv,
            hsv_tolerance: self.hsv_tolerance,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ColorDetection {
    label: String,
    pub colors: Vec<Color>,
    pub tolerance: u8,
}

impl ColorDetection {
    pub fn from(color: Vec<[u8; 3]>) -> Self {
        Self {
            colors: {
                let mut colors = vec![];
                for c in color {
                    colors.push(Color::new_rgb(c, None));
                }
                colors
            },
            tolerance: 5,
            label: "default".to_string(),
        }
    }

    pub fn from_hsv(color: Vec<[f32; 3]>, tolerance: Option<[f32; 3]>) -> Self {
        Self {
            colors: {
                let mut colors = vec![];
                for c in color {
                    colors.push(Color::new_hsv([c[0], c[1], c[2]], tolerance));
                }
                colors
            },
            tolerance: 5,
            label: "default".to_string(),
        }
    }

    pub fn set_label(&mut self, label: String) -> &mut Self {
        self.label = label;
        return self;
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
            if ref_color.match_pixel(*color, self.label.clone()) {
                return true;
            } else {
            }
        }
        false
    }
}

impl Clone for ColorDetection {
    fn clone(&self) -> Self {
        Self {
            colors: self.colors.clone(),
            tolerance: self.tolerance,
            label: self.label.clone(),
        }
    }
}

impl Default for ColorDetection {
    fn default() -> Self {
        Self {
            colors: vec![],
            tolerance: 0,
            label: "default".to_string(),
        }
    }
}
