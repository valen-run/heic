//! `iprp` (Item Properties Box), `ipco`, and `ipma` parser.

pub mod hvcc;
pub mod ipma;
pub mod properties;

use crate::boxes::{BoxHeader, BoxIter, FourCC};
use std::collections::HashMap;
use valen_heic_core::{HeicError, HeicResult, ImageDimensions, Limits};

pub use hvcc::parse_hvcc;
pub use ipma::{parse_ipma, PropertyAssociation};
pub use properties::{
    parse_ipco, AuxiliaryProperty, ColorProperty, HevcConfigProperty, ImageSpatialExtents,
    ItemProperty, MirrorProperty, RotationProperty,
};

/// Complete Item Properties Box (`iprp`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemPropertiesBox {
    /// Ordered list of properties from `ipco` (0-indexed internally, but `ipma` uses 1-based indexing).
    pub properties: Vec<ItemProperty>,
    /// Map of `item_id -> Vec<PropertyAssociation>`.
    pub associations: HashMap<u32, Vec<PropertyAssociation>>,
}

impl ItemPropertiesBox {
    /// Parses an `iprp` box from raw box bytes.
    pub fn parse(input: &[u8], limits: &Limits) -> HeicResult<Self> {
        let header = BoxHeader::parse(input)?;
        if header.box_type != FourCC::IPRP {
            return Err(HeicError::InvalidContainer(format!(
                "Expected 'iprp' box, got '{}'",
                header.box_type
            )));
        }

        let payload = header.payload(input)?;
        let mut iprp = Self::default();

        for res in BoxIter::new(payload) {
            let (child_header, child_data) = res?;
            match child_header.box_type {
                FourCC::IPCO => {
                    iprp.properties = parse_ipco(child_data, limits)?;
                }
                FourCC::IPMA => {
                    let map = parse_ipma(child_data, limits)?;
                    for (k, v) in map {
                        iprp.associations.entry(k).or_default().extend(v);
                    }
                }
                _ => {}
            }
        }

        Ok(iprp)
    }

    /// Retrieves all properties associated with a specific item ID in order.
    pub fn get_properties_for_item(&self, item_id: u32) -> Vec<&ItemProperty> {
        let mut result = Vec::new();
        if let Some(assocs) = self.associations.get(&item_id) {
            for assoc in assocs {
                if assoc.property_index > 0 && assoc.property_index <= self.properties.len() {
                    result.push(&self.properties[assoc.property_index - 1]);
                }
            }
        }
        result
    }

    /// Finds spatial extents (dimensions) for an item.
    pub fn get_dimensions_for_item(&self, item_id: u32) -> Option<ImageDimensions> {
        for prop in self.get_properties_for_item(item_id) {
            if let ItemProperty::SpatialExtents(ispe) = prop {
                return Some(ispe.to_dimensions());
            }
        }
        None
    }

    /// Finds rotation property for an item.
    pub fn get_rotation_for_item(&self, item_id: u32) -> Option<RotationProperty> {
        for prop in self.get_properties_for_item(item_id) {
            if let ItemProperty::Rotation(rot) = prop {
                return Some(*rot);
            }
        }
        None
    }

    /// Finds color property for an item.
    pub fn get_color_for_item(&self, item_id: u32) -> Option<&ColorProperty> {
        for prop in self.get_properties_for_item(item_id) {
            if let ItemProperty::Color(colr) = prop {
                return Some(colr);
            }
        }
        None
    }

    /// Finds HEVC decoder config record for an item.
    pub fn get_hevc_config_for_item(&self, item_id: u32) -> Option<&HevcConfigProperty> {
        for prop in self.get_properties_for_item(item_id) {
            if let ItemProperty::HevcConfig(hvc) = prop {
                return Some(hvc);
            }
        }
        None
    }

    /// Returns `true` if the item is declared as an auxiliary alpha transparency mask.
    pub fn is_alpha_mask_item(&self, item_id: u32) -> bool {
        for prop in self.get_properties_for_item(item_id) {
            if let ItemProperty::Auxiliary(aux) = prop {
                if aux.is_alpha() {
                    return true;
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rotation_property() {
        let rot0 = RotationProperty { angle_ccw: 0 };
        assert_eq!(rot0.angle_cw(), 0);
        assert_eq!(rot0.to_exif_orientation(), 1);

        let rot90_ccw = RotationProperty { angle_ccw: 1 };
        assert_eq!(rot90_ccw.angle_cw(), 270);
        assert_eq!(rot90_ccw.to_exif_orientation(), 8);

        let rot180 = RotationProperty { angle_ccw: 2 };
        assert_eq!(rot180.angle_cw(), 180);
        assert_eq!(rot180.to_exif_orientation(), 3);

        let rot270_ccw = RotationProperty { angle_ccw: 3 };
        assert_eq!(rot270_ccw.angle_cw(), 90);
        assert_eq!(rot270_ccw.to_exif_orientation(), 6);
    }
}
