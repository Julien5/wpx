    /*
        fn _import(filename: std::path::PathBuf) -> MapData {
            use svg::node::element::tag;
            use svg::parser::Event;
            let mut polyline = crate::svgmap::Polyline::new();
            let mut document = Attributes::new();
            let mut content = String::new();
            let mut points = Vec::new();
            let mut current_circle = PointFeature::new();
            let mut current_text_attributes = Attributes::new();
            for event in svg::open(filename, &mut content).unwrap() {
                match event {
                    Event::Tag(tag::Circle, _, attributes) => {
                        if attributes.contains_key("id") {
                            let id = attributes.get("id").unwrap().clone().to_string();
                            let (p_id, _p_attr) = _readid(id.as_str());
                            current_circle.id = String::from_str(p_id).unwrap();
                            current_circle.circle = Circle::_from_attributes(&attributes);
                        }
                    }
                    Event::Tag(tag::Text, _, attributes) => {
                        if attributes.contains_key("id") {
                            // let id = attributes.get("id").unwrap();
                            current_text_attributes = attributes.clone();
                        }
                    }
                    Event::Text(data) => {
                        current_circle.label = Label::_from_attributes(&current_text_attributes, data);
                        current_text_attributes.clear();
                        debug_assert!(!current_circle.id.is_empty());
                        points.push(current_circle);
                        current_circle = PointFeature::new();
                    }
                    Event::Tag(tag::Path, _, attributes) => {
                        if attributes.contains_key("id") {
                            let id = attributes.get("id").unwrap();
                        }
                        polyline = crate::svgmap::Polyline::_from_attributes(&attributes);
                        let data = attributes.get("d").unwrap();
                        let data = Data::parse(data).unwrap();
                        use svg::node::element::path::Command;
                        for command in data.iter() {
                            match command {
                                &Command::Move(..) => { /* … */ }
                                &Command::Line(..) => { /* … */ }
                                _ => {}
                            }
                        }
                    }
                    Event::Tag(tag::SVG, _, attributes) => {
                        if !attributes.is_empty() {
                            document = attributes.clone();
                        }
                    }
                    _ => {}
                }
            }

            MapData {
                polyline,
                points,
                document,
                debug: svg::node::element::Group::new(),
            }
    }
        */

