/****************************************************************************
**
** svgcleaner could help you to clean up your SVG files
** from unnecessary data.
** Copyright (C) 2012-2017 Evgeniy Reizner
**
** This program is free software; you can redistribute it and/or modify
** it under the terms of the GNU General Public License as published by
** the Free Software Foundation; either version 2 of the License, or
** (at your option) any later version.
**
** This program is distributed in the hope that it will be useful,
** but WITHOUT ANY WARRANTY; without even the implied warranty of
** MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
** GNU General Public License for more details.
**
** You should have received a copy of the GNU General Public License along
** with this program; if not, write to the Free Software Foundation, Inc.,
** 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
**
****************************************************************************/

use super::short::AId;

use svgdom::{Document, Attribute, AttributeType, AttributeValue, WriteOptions, WriteBuffer};

use options::StyleJoinMode;

pub fn join_style_attributes(doc: &Document, mode: StyleJoinMode, opt: &WriteOptions) {
    // NOTE: must be run at last, since it breaks linking.

    if mode == StyleJoinMode::None {
        return;
    }

    for node in doc.descendants().svg() {
        let count;
        {
            let attrs = node.attributes();
            count = attrs.iter().filter(|a| a.is_presentation() && a.visible).count();
        }

        // 5 - is an amount of attributes when style notation is becoming more efficient than
        // split attributes.
        if (mode == StyleJoinMode::Some && count > 5) || mode == StyleJoinMode::All {
            let mut attrs = node.attributes_mut();
            let mut ids = Vec::new();
            let mut style = Vec::new();
            for (aid, attr) in attrs.iter_svg().filter(|&(_, a)| a.is_presentation()) {
                if !attr.visible {
                    continue;
                }

                style.extend_from_slice(aid.name().as_bytes());
                style.push(b':');
                attr.value.write_buf_opt(opt, &mut style);
                style.push(b';');

                ids.push(aid);
            }
            style.pop();

            // unwrap can't fail
            let style_str = String::from_utf8(style).unwrap();
            attrs.insert(Attribute::new(AId::Style, AttributeValue::String(style_str)));

            for id in ids {
                attrs.remove(id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use svgdom::{Document};

    #[test]
    fn join_styles_1() {
        let doc = Document::from_str(
            "<svg fill='black' stroke='red' stroke-width='1' opacity='1' \
                  fill-opacity='1' stroke-opacity='1'/>"
        ).unwrap();

        let svg_node = doc.svg_element().unwrap();
        let wopt = WriteOptions::default();

        join_style_attributes(&doc, StyleJoinMode::None, &wopt);
        assert_eq!(svg_node.attribute(AId::Style), None);

        // we have 6 style attributes so they should be joined
        join_style_attributes(&doc, StyleJoinMode::Some, &wopt);
        assert_eq_text!(
            svg_node.attribute(AId::Style).unwrap().value.as_string().unwrap(),
            "fill:#000000;stroke:#ff0000;stroke-width:1;opacity:1;fill-opacity:1;stroke-opacity:1"
        );
    }

    #[test]
    fn join_styles_2() {
        let doc = Document::from_str(
            "<svg fill='black' stroke='red'/>"
        ).unwrap();

        let svg_node = doc.svg_element().unwrap();
        let wopt = WriteOptions::default();

        // we have only 2 style attributes so they shouldn't be joined
        join_style_attributes(&doc, StyleJoinMode::Some, &wopt);
        assert_eq!(svg_node.attribute(AId::Style), None);

        // join anyway
        join_style_attributes(&doc, StyleJoinMode::All, &wopt);
        assert_eq_text!(
            svg_node.attribute(AId::Style).unwrap().value.as_string().unwrap(),
            "fill:#000000;stroke:#ff0000"
        );
    }
}
