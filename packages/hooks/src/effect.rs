use tairitsu_vdom::{create_effect, EffectHandle};

pub type Effect = EffectHandle;

pub fn use_effect<F>(effect: F) -> Effect
where
    F: FnMut() + 'static,
{
    create_effect(effect)
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;

    #[test]
    fn test_use_effect() {
        let counter = Rc::new(RefCell::new(0));
        let counter_clone = Rc::clone(&counter);

        let _effect = use_effect(move || {
            *counter_clone.borrow_mut() += 1;
        });

        assert_eq!(*counter.borrow(), 1);
    }
}
