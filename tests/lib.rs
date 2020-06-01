

mod protomaps2d_tests {
    use protomaps2d::label::Collider;

    #[test]
    fn test_one() {
        let mut c = Collider{bboxes:Vec::new()};
        assert!(c.add((1.0,1.0),(3.0,3.0)));
        assert!(!c.add((2.0,2.0),(4.0,4.0)));
    }

    #[test]
    fn test_no_collision() {
        let mut c = Collider{bboxes:Vec::new()};
        assert!(c.add((3.0,3.0),(4.0,4.0)));

        // to the left
        assert!(c.add((1.0,3.0),(2.0,4.0)));

        // to the right
        assert!(c.add((5.0,3.0),(6.0,4.0)));

        // to the top
        assert!(c.add((3.0,1.0),(4.0,2.0)));

        // to the bottom
        assert!(c.add((3.0,5.0),(4.0,6.0)));
    }

}