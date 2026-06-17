use tairitsu_vdom::runtime::cleanup_component;

#[test]
fn test_cleanup_component_does_not_double_free() {
    let fake_id = 999999;
    cleanup_component(fake_id);
    cleanup_component(fake_id);
}
