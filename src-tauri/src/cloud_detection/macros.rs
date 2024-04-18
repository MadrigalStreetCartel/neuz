#[macro_export]
macro_rules! create_cloud_detection_kinds {
    ($name:ident, { $($variant:ident = ($array:expr, $tolerance:expr);)* }) => {
            use std::collections::HashMap;
            use std::fmt::Display;
            use lazy_static::lazy_static;
            use crate::data::ColorDetection;
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            pub enum $name {
                $($variant),*
            }

            impl $name {
                pub fn to_string(&self) -> &'static str {
                    match self {
                        $(Self::$variant => stringify!($variant)),*
                    }
                }

                pub fn get_colors(&self) -> ColorDetection {
                    COLORS_MAP.get(self.to_string()).expect("Color not found").clone()
                }
            }

            lazy_static! {
                pub static ref COLORS_MAP: HashMap<String, ColorDetection> = {
                    let mut m = HashMap::new();
                    $(m.insert(stringify!($variant).to_string(), ColorDetection::from_hsv($array.to_vec(), $tolerance).set_label(stringify!($variant).to_string()).to_owned());)*
                    m
                };
            }
            impl Display for $name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    self.to_string().fmt(f)
                }
            }
    };
}

#[macro_export]
macro_rules! create_cloud_detection_zones {
    ($name:ident, { $($variant:ident = $bounds:expr;)* }) => {
        use std::collections::HashMap;
        use lazy_static::lazy_static;
        use crate::data::Bounds;
        use crate::cloud_detection::CloudDetectionType;
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub enum $name {
            $($variant(Vec<CloudDetectionType>)),*,
            Custom(Bounds, Vec<CloudDetectionType>)
        }

        impl $name {
            pub fn to_string(&self) -> &'static str {
                match self {
                    $(Self::$variant(types) => stringify!($variant)),*,
                    Self::Custom(_, _) => "Custom"
                }
            }

            pub fn get_bounds(&self) -> Bounds {
                match self {
                    $(Self::$variant(types) => ZONES_MAP.get(stringify!($variant)).expect("Zone not found").clone()),*,
                    Self::Custom(bounds, _) => bounds.clone()
                }
            }

            pub fn get_types(&self) -> Vec<CloudDetectionType> {
                match self {
                    $(Self::$variant(types) => types.to_vec()),*,
                    Self::Custom(_, types) => types.clone()
                }
            }

            pub fn is(&self, name: &str) -> bool {
                self.to_string() == name
            }
        }

        lazy_static! {
            pub static ref ZONES_MAP: HashMap<String, Bounds> = {
                let mut m = HashMap::new();
                $(m.insert(stringify!($variant).to_string(), $bounds);)*
                m
            };
        }
    };
}
