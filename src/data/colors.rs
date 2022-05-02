use std::{str::FromStr, fmt::{Formatter, Display}};

use serde::{Deserialize, Serialize, Deserializer, de::Visitor};
use serde_yaml::Value;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b))
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // code from https://github.com/alacritty/alacritty/blob/master/alacritty_terminal/src/term/color.rs
        struct ColorVisitor;

        // Used for deserializing reftests.
        #[derive(Deserialize)]
        struct ColorDerivedDeser {
            r: u8,
            g: u8,
            b: u8,
        }

        impl<'a> Visitor<'a> for ColorVisitor {
            type Value = Color;

            fn expecting(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                f.write_str("hex color like #ff00ff")
            }

            fn visit_str<E>(self, value: &str) -> Result<Color, E>
            where
                E: serde::de::Error,
            {
                Color::from_str(value).map_err(|_| {
                    E::custom(format!(
                        "failed to parse rgb color {}; expected hex color like #ff00ff",
                        value
                    ))
                })
            }
        }

        // Return an error if the syntax is incorrect.
        let value = Value::deserialize(deserializer)?;

        // Attempt to deserialize from struct form.
        if let Ok(ColorDerivedDeser { r, g, b }) = ColorDerivedDeser::deserialize(value.clone()) {
            return Ok(Color { r, g, b });
        }

        // Deserialize from hex notation (either 0xff00ff or #ff00ff).
        value.clone().deserialize_str(ColorVisitor).map_err(|_| {
            serde::de::Error::custom(format!(
                "failed to parse rgb color {}; expected hex color like #ff00ff",
                value.as_str().unwrap_or("<null>")
            ))
        })
    }

    fn deserialize_in_place<D>(deserializer: D, place: &mut Self) -> Result<(), D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Default implementation just delegates to `deserialize` impl.
        *place = Self::deserialize(deserializer)?;
        Ok(())
    }
}

impl FromStr for Color {
    type Err = ();

    fn from_str(s: &str) -> Result<Color, ()> {
        let chars = if s.starts_with("0x") && s.len() == 8 {
            &s[2..]
        } else if s.starts_with('#') && s.len() == 7 {
            &s[1..]
        } else {
            return Err(());
        };

        match u32::from_str_radix(chars, 16) {
            Ok(mut color) => {
                let b = (color & 0xff) as u8;
                color >>= 8;
                let g = (color & 0xff) as u8;
                color >>= 8;
                let r = color as u8;
                Ok(Color { r, g, b })
            },
            Err(_) => Err(()),
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

impl Default for Color {
    fn default() -> Self {
        Color {
            r: 255,
            g: 255,
            b: 255,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LinkColor {
    pub normal: Color,
    pub hover: Color,
}



impl Default for LinkColor {
    fn default() -> Self {
        Self { normal: Default::default(), hover: Default::default() }
    }
}

impl Display for LinkColor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ normal: {}, hover: {} }}", self.normal, self.hover)
    }
}
