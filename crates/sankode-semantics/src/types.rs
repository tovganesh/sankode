use sankode_core::TypeAnnotation;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Purna8,
    Purna16,
    Purna32,
    Purna64,
    Apu8,
    Apu16,
    Apu32,
    Apu64,
    Ansha32,
    Ansha64,
    Dvaidha,
    Varna,
    Sutra, // Owned heap string: Move semantics (non-Copy)!
    Rikta,
    Struct(String),
    Reference(Box<Type>),
    MutReference(Box<Type>),
    Unknown,
}

impl Type {
    /// Returns true if this type satisfies Copy semantics (like Rust primitive scalars)
    pub fn is_copy(&self) -> bool {
        match self {
            Type::Purna8
            | Type::Purna16
            | Type::Purna32
            | Type::Purna64
            | Type::Apu8
            | Type::Apu16
            | Type::Apu32
            | Type::Apu64
            | Type::Ansha32
            | Type::Ansha64
            | Type::Dvaidha
            | Type::Varna
            | Type::Rikta
            | Type::Reference(_) => true,
            // Sutra, Struct, and MutReference are non-Copy (affine move semantics)
            Type::Sutra | Type::Struct(_) | Type::MutReference(_) | Type::Unknown => false,
        }
    }

    pub fn from_annotation(ann: &TypeAnnotation) -> Self {
        match ann {
            TypeAnnotation::Simple(name) => match name.as_str() {
                "पूर्ण८" => Type::Purna8,
                "पूर्ण१६" => Type::Purna16,
                "पूर्ण३२" => Type::Purna32,
                "पूर्ण६४" | "पूर्ण" => Type::Purna64,
                "अपू८" => Type::Apu8,
                "अपू१६" => Type::Apu16,
                "अपू३२" => Type::Apu32,
                "अपू६४" => Type::Apu64,
                "अंश३२" => Type::Ansha32,
                "अंश६४" | "अंश" => Type::Ansha64,
                "द्वैध" => Type::Dvaidha,
                "वर्ण" => Type::Varna,
                "सूत्र" => Type::Sutra,
                "रिक्त" => Type::Rikta,
                other => Type::Struct(other.to_string()),
            },
            TypeAnnotation::Reference(inner) => {
                Type::Reference(Box::new(Self::from_annotation(inner)))
            }
            TypeAnnotation::MutReference(inner) => {
                Type::MutReference(Box::new(Self::from_annotation(inner)))
            }
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Purna8 => write!(f, "पूर्ण८"),
            Type::Purna16 => write!(f, "पूर्ण१६"),
            Type::Purna32 => write!(f, "पूर्ण३२"),
            Type::Purna64 => write!(f, "पूर्ण६४"),
            Type::Apu8 => write!(f, "अपू८"),
            Type::Apu16 => write!(f, "अपू१६"),
            Type::Apu32 => write!(f, "अपू३२"),
            Type::Apu64 => write!(f, "अपू६४"),
            Type::Ansha32 => write!(f, "अंश३२"),
            Type::Ansha64 => write!(f, "अंश६४"),
            Type::Dvaidha => write!(f, "द्वैध"),
            Type::Varna => write!(f, "वर्ण"),
            Type::Sutra => write!(f, "सूत्र"),
            Type::Rikta => write!(f, "रिक्त"),
            Type::Struct(name) => write!(f, "{}", name),
            Type::Reference(inner) => write!(f, "ऋण {}", inner),
            Type::MutReference(inner) => write!(f, "चलऋण {}", inner),
            Type::Unknown => write!(f, "अज्ञात"),
        }
    }
}
