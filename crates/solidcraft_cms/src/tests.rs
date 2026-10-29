//! Unit tests for SolidCraft CMS truFold data models and edge-tracing algorithms.

#[cfg(test)]
mod tests {
    use crate::data::*;
    use crate::trace::*;

    #[test]
    fn test_cms_metadata_serialization_roundtrip() {
        let mut meta = CmsMetadata::new(
            CmsProductType::UnequalTee,
            "BONN-CT-TEE-300X150",
            1.5,
        );

        meta.cms_ports.push(CmsPortMetadata::new(
            1,
            "P1 - Main Run In",
            300.0,
            50.0,
            (-1.0, 0.0),
            (-150.0, 0.0),
        ));
        meta.cms_ports.push(CmsPortMetadata::new(
            2,
            "P2 - Main Run Out",
            300.0,
            50.0,
            (1.0, 0.0),
            (150.0, 0.0),
        ));
        meta.cms_ports.push(CmsPortMetadata::new(
            3,
            "P3 - Branch",
            150.0,
            50.0,
            (0.0, 1.0),
            (0.0, 200.0),
        ));

        meta.rigid_slots.push(CmsSlotMetadata::new(
            1,
            vec![(-140.0, -10.0), (-120.0, -10.0), (-120.0, 10.0), (-140.0, 10.0)],
            (-130.0, 0.0),
            1,
            (20.0, 0.0),
        ));

        meta.active_bends.push(CmsFoldParams::new(
            "fold_1",
            1,
            (-100.0, -50.0),
            (100.0, -50.0),
            1.5,
            90.0,
            BendDirection::Up,
        ));

        let json = serde_json::to_string_pretty(&meta).expect("Serialization failed");
        assert!(json.contains("UnequalTee"));
        assert!(json.contains("BONN-CT-TEE-300X150"));

        let deserialized: CmsMetadata = serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(deserialized.product_classification, CmsProductType::UnequalTee);
        assert_eq!(deserialized.cms_ports.len(), 3);
        assert_eq!(deserialized.rigid_slots.len(), 1);
        assert_eq!(deserialized.active_bends.len(), 1);
        assert_eq!(deserialized.active_bends[0].direction, BendDirection::Up);
    }

    #[test]
    fn test_trace_dxf_boundaries_clean_rectangle() {
        let segments = vec![
            RawDxfSegment::new((0.0, 0.0), (100.0, 0.0)),
            RawDxfSegment::new((100.0, 0.0), (100.0, 50.0)),
            RawDxfSegment::new((100.0, 50.0), (0.0, 50.0)),
            RawDxfSegment::new((0.0, 50.0), (0.0, 0.0)),
        ];

        let loops = trace_dxf_boundaries(&segments, 0.05);
        assert_eq!(loops.len(), 1);
        assert_eq!(loops[0].vertices.len(), 4);
        assert!((loops[0].area() - 5000.0).abs() < 1e-4);
        assert_eq!(loops[0].centroid(), (50.0, 25.0));
    }

    #[test]
    fn test_trace_dxf_boundaries_unordered_with_microgaps() {
        // Scrambled order with reversed segments and 0.02mm micro-gaps (simulating real CNC laser DXFs)
        let segments = vec![
            RawDxfSegment::new((100.015, 50.0), (0.0, 50.01)),   // top edge (reversed)
            RawDxfSegment::new((0.0, 0.0), (100.0, 0.0)),        // bottom edge
            RawDxfSegment::new((0.0, 0.01), (0.0, 50.01)),       // left edge
            RawDxfSegment::new((100.0, 0.0), (100.015, 50.0)),   // right edge
        ];

        let loops = trace_dxf_boundaries(&segments, 0.05);
        assert_eq!(loops.len(), 1, "Should successfully bridge <= 0.05mm gaps");
        assert_eq!(loops[0].vertices.len(), 4);
        assert!((loops[0].area() - 5000.0).abs() < 10.0);
    }

    #[test]
    fn test_isolate_outer_perimeter_and_hole() {
        // Outer blank: 400 x 200 mm
        let outer_segs = vec![
            RawDxfSegment::new((0.0, 0.0), (400.0, 0.0)),
            RawDxfSegment::new((400.0, 0.0), (400.0, 200.0)),
            RawDxfSegment::new((400.0, 200.0), (0.0, 200.0)),
            RawDxfSegment::new((0.0, 200.0), (0.0, 0.0)),
        ];

        // Interior slot: 20 x 8 mm obround cutout
        let slot_segs = vec![
            RawDxfSegment::new((50.0, 50.0), (70.0, 50.0)),
            RawDxfSegment::new((70.0, 50.0), (70.0, 58.0)),
            RawDxfSegment::new((70.0, 58.0), (50.0, 58.0)),
            RawDxfSegment::new((50.0, 58.0), (50.0, 50.0)),
        ];

        let mut all_segs = outer_segs;
        all_segs.extend(slot_segs);

        let discovered_loops = trace_dxf_boundaries(&all_segs, 0.05);
        assert_eq!(discovered_loops.len(), 2);

        let (outer, holes) = isolate_outer_perimeter(discovered_loops);
        assert!(outer.is_some());
        let outer_loop = outer.unwrap();
        assert!((outer_loop.area() - 80000.0).abs() < 1e-4);
        assert_eq!(holes.len(), 1);
        assert!((holes[0].area() - 160.0).abs() < 1e-4);
    }

    #[test]
    fn test_find_open_perimeter_ports() {
        // 300x200 rectangle representing a straight tray section
        let loop_wire = StructuralWireLoop::new(
            vec![0, 1, 2, 3],
            vec![(0.0, 0.0), (300.0, 0.0), (300.0, 200.0), (0.0, 200.0)],
        );

        // Find edges with span >= 100mm
        let ports = loop_wire.find_open_perimeter_ports(100.0);
        assert_eq!(ports.len(), 4);

        // Check left edge: (0, 200) -> (0, 0) has mid at (0, 100), outward normal should point (-1, 0)
        let left_port = ports.iter().find(|p| (p.center.0 - 0.0).abs() < 1e-4 && (p.center.1 - 100.0).abs() < 1e-4).expect("Left port");
        assert!((left_port.outward_normal.0 - (-1.0)).abs() < 1e-4);
        assert!((left_port.outward_normal.1 - 0.0).abs() < 1e-4);

        // Check right edge: (300, 0) -> (300, 200) has mid at (300, 100), outward normal should point (+1, 0)
        let right_port = ports.iter().find(|p| (p.center.0 - 300.0).abs() < 1e-4 && (p.center.1 - 100.0).abs() < 1e-4).expect("Right port");
        assert!((right_port.outward_normal.0 - 1.0).abs() < 1e-4);
        assert!((right_port.outward_normal.1 - 0.0).abs() < 1e-4);
    }

    #[test]
    fn test_classify_reducer() {
        // Concentric reducer flat profile:
        // P1 at x=0 (span 300mm: y=-150 to y=150)
        // Slanted top flank to x=200, y=75
        // P2 at x=200 (span 150mm: y=75 to y=-75)
        // Slanted bottom flank back to x=0, y=-150
        let outer_loop = StructuralWireLoop::new(
            vec![0, 1, 2, 3],
            vec![
                (0.0, -150.0),
                (0.0, 150.0),
                (200.0, 75.0),
                (200.0, -75.0),
            ],
        );

        let meta = crate::topology::cms_identify_topology(&outer_loop, &[], 50.0, 1.5)
            .expect("Classification failed");

        assert_eq!(meta.product_classification, CmsProductType::Reducer);
        assert!(meta.product_name.starts_with("CMS-REDUCER"));
        assert_eq!(meta.cms_ports.len(), 2);
        assert!((meta.cms_ports[0].base_width - 300.0).abs() < 1.0);
        assert!((meta.cms_ports[1].base_width - 150.0).abs() < 1.0);
        assert!(!meta.cms_ports[0].vertex_indices.is_empty());
    }

    #[test]
    fn test_classify_unequal_tee() {
        // T-junction flat outline with Main In (-150 to 150 along y at x=-200),
        // Main Out (-150 to 150 along y at x=200), and Branch (-75 to 75 along x at y=250)
        let outer_loop = StructuralWireLoop::new(
            vec![0, 1, 2, 3, 4, 5],
            vec![
                (-200.0, -150.0),
                (-200.0, 150.0),
                (-75.0, 250.0),
                (75.0, 250.0),
                (200.0, 150.0),
                (200.0, -150.0),
            ],
        );

        let meta = crate::topology::cms_identify_topology(&outer_loop, &[], 50.0, 1.5)
            .expect("Classification failed");

        assert_eq!(meta.product_classification, CmsProductType::UnequalTee);
        assert_eq!(meta.cms_ports.len(), 3);
        assert!(meta.product_name.contains("TEE"));
    }

    #[test]
    fn test_classify_90_deg_bend() {
        // 90° elbow with 2 orthogonal ports of equal span (200mm)
        // Port 1 at x=0 (y: 0 to 200), normal = (-1, 0)
        // Port 2 at y=0 (x: 0 to 200), normal = (0, -1)
        // Corner and outer bend polygon
        let outer_loop = StructuralWireLoop::new(
            vec![0, 1, 2, 3],
            vec![
                (0.0, 0.0),
                (0.0, 200.0),
                (300.0, 300.0),
                (200.0, 0.0),
            ],
        );

        let meta = crate::topology::cms_identify_topology(&outer_loop, &[], 50.0, 1.5)
            .expect("Classification failed");

        assert_eq!(meta.product_classification, CmsProductType::StandardBendOrRiser);
        assert_eq!(meta.cms_ports.len(), 2);
        assert!(meta.product_name.contains("BEND"));
    }

    #[test]
    fn test_coupler_slot_tagging_and_anchoring() {
        let outer_loop = StructuralWireLoop::new(
            vec![0, 1, 2, 3],
            vec![(0.0, 0.0), (300.0, 0.0), (300.0, 200.0), (0.0, 200.0)],
        );

        // Standard coupler obround: 25 x 9 mm, near x=0 edge
        let slot_loop = StructuralWireLoop::new(
            vec![0, 1, 2, 3],
            vec![(15.0, 45.5), (40.0, 45.5), (40.0, 54.5), (15.0, 54.5)],
        );

        let meta = crate::topology::cms_identify_topology(&outer_loop, &[slot_loop], 50.0, 1.5)
            .expect("Classification failed");

        assert_eq!(meta.rigid_slots.len(), 1);
        let slot = &meta.rigid_slots[0];
        assert!(slot.is_coupler_slot);
        assert_eq!(slot.slot_pattern.as_deref(), Some("OBROUND_25X9"));
        assert!(!slot.contour.is_empty());
    }

    #[test]
    fn test_classify_4way_cross() {
        // Cross intersection with 4 symmetrical ports of 200mm span:
        // P1: x = -200, y from -100 to 100 (normal = -1, 0)
        // P2: x = +200, y from -100 to 100 (normal = +1, 0)
        // P3: y = +200, x from -100 to 100 (normal = 0, +1)
        // P4: y = -200, x from -100 to 100 (normal = 0, -1)
        let outer_loop = StructuralWireLoop::new(
            vec![0, 1, 2, 3, 4, 5, 6, 7],
            vec![
                (-200.0, -100.0),
                (-200.0, 100.0),
                (-100.0, 200.0),
                (100.0, 200.0),
                (200.0, 100.0),
                (200.0, -100.0),
                (100.0, -200.0),
                (-100.0, -200.0),
            ],
        );

        let meta = crate::topology::cms_identify_topology(&outer_loop, &[], 50.0, 1.5)
            .expect("Classification failed");

        assert_eq!(meta.product_classification, CmsProductType::UnequalCross);
        assert_eq!(meta.cms_ports.len(), 4);
        assert!(meta.product_name.contains("CROSS"));
    }

    // =========================================================================
    // TASK 4: 2D PARAMETRIC PRE-PROCESSOR & PIN-RIGID MIGRATION TESTS
    // =========================================================================

    #[test]
    fn test_morph_base_2d_sketch_reducer() {
        // Reducer with P1 (width 300) and P2 (width 150)
        let mut outer_loop = StructuralWireLoop::new(
            vec![0, 1, 2, 3],
            vec![
                (0.0, -150.0), // v0: P1 bottom
                (0.0, 150.0),  // v1: P1 top
                (200.0, 75.0), // v2: P2 top
                (200.0, -75.0),// v3: P2 bottom
            ],
        );

        let mut meta = crate::topology::cms_identify_topology(&outer_loop, &[], 50.0, 1.5)
            .expect("Classification failed");

        // Scale P1 from 300mm to 400mm (+100mm expansion)
        meta.cms_ports[0].current_width = 400.0;
        // Keep P2 at 150mm
        meta.cms_ports[1].current_width = 150.0;

        crate::eval::morph_base_2d_sketch(&mut outer_loop, &meta);

        // P1 should now span 400mm: y from -200 to +200
        assert!((outer_loop.vertices[0].1 - (-200.0)).abs() < 1e-4);
        assert!((outer_loop.vertices[1].1 - 200.0).abs() < 1e-4);
        assert!((outer_loop.vertices[0].0 - 0.0).abs() < 1e-4);
        assert!((outer_loop.vertices[1].0 - 0.0).abs() < 1e-4);

        // P2 should remain unchanged: y from -75 to +75
        assert!((outer_loop.vertices[2].1 - 75.0).abs() < 1e-4);
        assert!((outer_loop.vertices[3].1 - (-75.0)).abs() < 1e-4);
        assert!((outer_loop.vertices[2].0 - 200.0).abs() < 1e-4);
        assert!((outer_loop.vertices[3].0 - 200.0).abs() < 1e-4);
    }

    #[test]
    fn test_migrate_rigid_coupler_slots() {
        // Outer loop with anchor edge 0 at x = 0 (y: -150 to 150, mid = (0, 0))
        let mut outer_loop = StructuralWireLoop::new(
            vec![0, 1, 2, 3],
            vec![
                (0.0, -150.0),
                (0.0, 150.0),
                (200.0, 150.0),
                (200.0, -150.0),
            ],
        );

        // Rigid obround slot 20x10 centered at (25.0, 0.0) relative to anchor mid (0, 0)
        let mut slots = vec![CmsSlotMetadata::new(
            1,
            vec![
                (15.0, -5.0),
                (35.0, -5.0),
                (35.0, 5.0),
                (15.0, 5.0),
            ],
            (25.0, 0.0),
            0, // anchored to edge 0
            (25.0, 0.0), // relative offset
        )];

        // Simulate outer loop shifting/expanding: edge 0 moves by (+10, +20)
        outer_loop.vertices[0] = (10.0, -130.0);
        outer_loop.vertices[1] = (10.0, 170.0);
        // New midpoint of edge 0 is (10.0, 20.0)

        crate::eval::migrate_rigid_coupler_slots(&mut slots, &outer_loop);

        let migrated = &slots[0];
        // Target center = new anchor mid (10.0, 20.0) + offset (25.0, 0.0) = (35.0, 20.0)
        assert!((migrated.center.0 - 35.0).abs() < 1e-4);
        assert!((migrated.center.1 - 20.0).abs() < 1e-4);

        // Verify contour translated rigidly without oval distortion (width must be exactly 20, height 10)
        let width = migrated.contour[1].0 - migrated.contour[0].0;
        let height = migrated.contour[2].1 - migrated.contour[1].1;
        assert!((width - 20.0).abs() < 1e-4, "Hole width preserved rigidly");
        assert!((height - 10.0).abs() < 1e-4, "Hole height preserved rigidly");
    }

    #[test]
    fn test_reposition_bend_lines() {
        let outer_loop = StructuralWireLoop::new(
            vec![0, 1, 2, 3],
            vec![
                (0.0, -150.0),
                (0.0, 150.0),
                (200.0, 75.0),
                (200.0, -75.0),
            ],
        );

        let mut meta = crate::topology::cms_identify_topology(&outer_loop, &[], 50.0, 1.5)
            .expect("Classification failed");

        // Expand Port 1 from 300 to 400 (+100mm)
        meta.cms_ports[0].current_width = 400.0;

        // Create two bend lines parallel to the port's run (along x axis)
        let mut bends = vec![
            CmsFoldParams::new("b1", 1, (10.0, 60.0), (100.0, 60.0), 1.5, 90.0, BendDirection::Up),
            CmsFoldParams::new("b2", 2, (10.0, -60.0), (100.0, -60.0), 1.5, 90.0, BendDirection::Up),
        ];

        crate::eval::reposition_bend_lines(&mut bends, &outer_loop, &meta);

        // Lateral expansion (+50 on top, -50 on bottom)
        assert!((bends[0].hinge_p1.1 - 110.0).abs() < 1e-4);
        assert!((bends[0].hinge_p2.1 - 110.0).abs() < 1e-4);
        assert!((bends[1].hinge_p1.1 - (-110.0)).abs() < 1e-4);
        assert!((bends[1].hinge_p2.1 - (-110.0)).abs() < 1e-4);
    }

    // =========================================================================
    // TASK 5: RELAXED B-REP SOLID PRIMITIVE FOLDS & FILLET ASSEMBLY TESTS
    // =========================================================================

    #[test]
    fn test_transform_3d_rotations_and_translations() {
        let t = crate::eval::Transform3D::translation(10.0, 20.0, 30.0);
        let p = t.transform_point3([0.0, 0.0, 0.0]);
        assert_eq!(p, [10.0, 20.0, 30.0]);

        // 90 degrees around Z axis: (1, 0, 0) -> (0, 1, 0)
        let rot_z = crate::eval::Transform3D::rotation_axis_angle([0.0, 0.0, 1.0], std::f64::consts::FRAC_PI_2);
        let p_rot = rot_z.transform_point3([1.0, 0.0, 0.0]);
        assert!((p_rot[0] - 0.0).abs() < 1e-6);
        assert!((p_rot[1] - 1.0).abs() < 1e-6);
        assert!((p_rot[2] - 0.0).abs() < 1e-6);

        // Combined translation and rotation
        let composed = t.mul(&rot_z);
        let p_comp = composed.transform_point3([1.0, 0.0, 0.0]);
        assert!((p_comp[0] - 10.0).abs() < 1e-6);
        assert!((p_comp[1] - 21.0).abs() < 1e-6);
        assert!((p_comp[2] - 30.0).abs() < 1e-6);
    }

    #[test]
    fn test_split_wire_loop_by_line() {
        let square = StructuralWireLoop::new(
            vec![0, 1, 2, 3],
            vec![
                (0.0, 0.0),
                (100.0, 0.0),
                (100.0, 100.0),
                (0.0, 100.0),
            ],
        );

        // Split across x = 50 from (50, 0) to (50, 100)
        let left = crate::eval::split_wire_loop_by_line(&square, (50.0, 0.0), (50.0, 100.0), true);
        let right = crate::eval::split_wire_loop_by_line(&square, (50.0, 0.0), (50.0, 100.0), false);

        assert!((left.area() - 5000.0).abs() < 1e-3, "Left half should be 5000 mm^2");
        assert!((right.area() - 5000.0).abs() < 1e-3, "Right half should be 5000 mm^2");
    }

    #[test]
    fn test_execute_relaxed_fold_90_degrees() {
        let sheet = StructuralWireLoop::new(
            vec![0, 1, 2, 3],
            vec![
                (0.0, 0.0),
                (300.0, 0.0),
                (300.0, 100.0),
                (0.0, 100.0),
            ],
        );

        let fold = CmsFoldParams::new(
            "fold_side",
            1,
            (100.0, 0.0),
            (100.0, 100.0),
            2.0,
            90.0,
            BendDirection::Up,
        );

        let result = crate::eval::execute_relaxed_fold(&fold, &sheet, 1.5);

        // Stationary base should have points with z = 0
        for pt in &result.stationary_vertices {
            assert!((pt[2] - 0.0).abs() < 1e-6);
        }

        // Rotated flange should have rotated out of the z=0 plane
        let has_nonzero_z = result.rotated_flange_vertices.iter().any(|pt| pt[2].abs() > 1e-3);
        assert!(has_nonzero_z, "Flange must rotate out of z=0 plane");

        // Fillet quad mesh generated
        assert_eq!(result.fillet_triangles.len(), 16); // 8 steps * 2 triangles
        assert!(!result.fused_solid_triangles.is_empty());
    }

    #[test]
    fn test_execute_relaxed_fold_miter_safety() {
        // Blank with sharp 45-degree miter corner cutout that would crash standard B-rep kernels
        let sheet = StructuralWireLoop::new(
            vec![0, 1, 2, 3, 4],
            vec![
                (0.0, 0.0),
                (150.0, 0.0),
                (200.0, 50.0), // sharp 45 miter notch
                (150.0, 100.0),
                (0.0, 100.0),
            ],
        );

        let fold = CmsFoldParams::new(
            "miter_fold",
            1,
            (80.0, 0.0),
            (80.0, 100.0),
            1.5,
            90.0,
            BendDirection::Up,
        );

        // Execute relaxed fold without B-Rep non-manifold crash
        let result = crate::eval::execute_relaxed_fold(&fold, &sheet, 1.5);
        assert!(!result.fused_solid_triangles.is_empty());
        assert!(!result.rotated_flange_vertices.is_empty());
    }

    // =========================================================================
    // TASK 7: MANUFACTURING EXPORTER & BATCH AUTOMATION TESTS
    // =========================================================================

    #[test]
    fn test_generate_2d_flat_dxf_layer_segregation() {
        let outer = StructuralWireLoop::new(
            vec![0, 1, 2, 3],
            vec![(0.0, 0.0), (300.0, 0.0), (300.0, 100.0), (0.0, 100.0)],
        );
        let slot = CmsSlotMetadata::new(
            1,
            vec![(10.0, 10.0), (30.0, 10.0), (30.0, 20.0), (10.0, 20.0)],
            (20.0, 15.0),
            0,
            (20.0, 15.0),
        );
        let bend = CmsFoldParams::new(
            "b1",
            1,
            (0.0, 50.0),
            (300.0, 50.0),
            1.5,
            90.0,
            BendDirection::Up,
        );

        let options = crate::export::DxfExportOptions::default();
        let dxf = crate::export::generate_2d_flat_dxf_string(&outer, &[slot], &[bend], &options);

        // Header check
        assert!(dxf.contains("AC1009"), "AutoCAD R12 header");
        assert!(dxf.contains("$INSUNITS"), "Units variable");

        // Layers table
        assert!(dxf.contains("CMS_FLAT_OUTLINE"), "Outer outline layer declared");
        assert!(dxf.contains("CMS_PUNCH_HOLES"), "Punch holes layer declared");
        assert!(dxf.contains("CMS_BEND_LINES"), "Bend lines layer declared");

        // Entity assignments
        assert!(dxf.contains("LINE\n8\nCMS_FLAT_OUTLINE"), "Perimeter lines assigned to outline layer");
        assert!(dxf.contains("LINE\n8\nCMS_PUNCH_HOLES"), "Punch slot lines assigned to punch layer");
        assert!(dxf.contains("LINE\n8\nCMS_BEND_LINES"), "Hinge lines assigned to bend layer");
    }

    #[test]
    fn test_generate_3d_step_structure() {
        let sheet = StructuralWireLoop::new(
            vec![0, 1, 2, 3],
            vec![(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)],
        );
        let fold = CmsFoldParams::new(
            "b1",
            1,
            (50.0, 0.0),
            (50.0, 100.0),
            1.5,
            90.0,
            BendDirection::Up,
        );
        let result = crate::eval::execute_relaxed_fold(&fold, &sheet, 1.5);

        let step = crate::export::generate_3d_step_string(&result, "TEST_PART");

        assert!(step.contains("ISO-10303-21"), "Valid ISO 10303-21 header");
        assert!(step.contains("CONFIG_CONTROL_DESIGN"), "AP203/AP214 schema");
        assert!(step.contains("CARTESIAN_POINT"), "Cartesian vertex definitions");
        assert!(step.contains("POLY_LOOP"), "Triangle poly loops");
        assert!(step.contains("CLOSED_SHELL"), "Solid shell closure");
        assert!(step.contains("FACETED_BREP"), "Faceted B-Rep solid model");
        assert!(step.contains("END-ISO-10303-21"), "Valid STEP footer");
    }

    #[test]
    fn test_parse_cms_order_csv() {
        let csv_data = r#"
Item_Code,Category,Product,Variation,Width,Height,Thickness,Secondary_Width,Angle,Direction
BONN-CT-RED-300-150,Cable Tray,Reducer,Concentric,300,50,1.5,150,90,UP
BONN-CT-TEE-300-100,Cable Tray,UnequalTee,Standard,300,50,1.5,100,90,UP
BONN-CT-BEND-90-200,Cable Tray,Bend,90Deg,200,50,1.5,,90,UP
"#;

        let rows = crate::batch::parse_cms_order_csv(csv_data).expect("CSV parsing failed");
        assert_eq!(rows.len(), 3);

        // Row 1
        assert_eq!(rows[0].item_code, "BONN-CT-RED-300-150");
        assert_eq!(rows[0].product_type, CmsProductType::Reducer);
        assert_eq!(rows[0].width, 300.0);
        assert_eq!(rows[0].secondary_width, Some(150.0));
        assert_eq!(rows[0].height, 50.0);
        assert_eq!(rows[0].thickness, 1.5);

        // Row 2
        assert_eq!(rows[1].item_code, "BONN-CT-TEE-300-100");
        assert_eq!(rows[1].product_type, CmsProductType::UnequalTee);
        assert_eq!(rows[1].secondary_width, Some(100.0));

        // Row 3 (deduced from name)
        assert_eq!(rows[2].item_code, "BONN-CT-BEND-90-200");
        assert_eq!(rows[2].product_type, CmsProductType::StandardBendOrRiser);
    }

    #[test]
    fn test_execute_cms_batch_processing_headless() {
        let csv_data = r#"
Item_Code,Category,Product,Variation,Width,Height,Thickness
BATCH-TEST-RED-400-200,Cable Tray,Reducer,Concentric,400,50,1.5,200
BATCH-TEST-TEE-300,Cable Tray,UnequalTee,Standard,300,50,1.5
"#;

        let temp_dir = std::env::temp_dir().join("cms_batch_test_out");
        let report = crate::batch::execute_cms_batch_processing(csv_data, &temp_dir)
            .expect("Batch execution failed");

        assert_eq!(report.total_rows, 2);
        assert_eq!(report.successful_items.len(), 2);
        assert!(report.failed_rows.is_empty());

        for item in &report.successful_items {
            assert!(item.flat_dxf_path.exists(), "Flat DXF file must exist");
            assert!(item.folded_step_path.exists(), "3D STEP file must exist");

            let dxf_size = std::fs::metadata(&item.flat_dxf_path).unwrap().len();
            let step_size = std::fs::metadata(&item.folded_step_path).unwrap().len();
            assert!(dxf_size > 100, "DXF file non-empty");
            assert!(step_size > 100, "STEP file non-empty");
        }

        // Clean up temp test directory
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}


