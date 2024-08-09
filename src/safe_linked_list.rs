use ghost_cell::{GhostCell, GhostToken};
use static_rc::StaticRc;

pub struct SafeLinkedList<'id, T> {
    head: Option<Half<'id, T>>,
    tail: Option<Half<'id, T>>,
    len: usize,
}

struct Node<'id, T> {
    elem: T,
    next: Option<Half<'id, T>>,
    prev: Option<Half<'id, T>>,
}

type Half<'id, T> = StaticRc<GhostNode<'id, T>, 1, 2>;
type Full<'id, T> = StaticRc<GhostNode<'id, T>, 2, 2>;

type GhostNode<'id, T> = GhostCell<'id, Node<'id, T>>;

impl<'id, T> Node<'id, T> {
    fn with_elem(elem: T) -> Self {
        Self {
            elem,
            next: None,
            prev: None,
        }
    }
}

impl<'id, T> SafeLinkedList<'id, T> {
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
            len: 0,
        }
    }

    pub fn push_back(&mut self, elem: T, token: &mut GhostToken<'id>) {
        let (one, two) = Full::split::<1, 1>(Full::new(GhostCell::new(Node::with_elem(elem))));
        if let Some(left) = self.tail.take() {
            left.borrow_mut(token).next = Some(one);
            two.borrow_mut(token).prev = Some(left);
            self.tail = Some(two);
        } else {
            self.head = Some(one);
            self.tail = Some(two);
        }
        self.len += 1;
    }

    pub fn pop_back(&mut self, token: &mut GhostToken<'id>) -> Option<T> {
        let two = self.tail.take()?;
        let one = if let Some(left) = two.borrow_mut(token).prev.take() {
            let one = left.borrow_mut(token).next.take().unwrap();
            self.tail = Some(left);
            one
        } else {
            self.head.take().unwrap()
        };
        // GhostCell 无法移动，而 Full::into_inner 可以在不移动的情况下拿出内部所有权
        Some(Full::into_inner(Full::join(one, two)).into_inner().elem)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_and_pop_back() {
        GhostToken::new(|mut token| {
            let mut list = SafeLinkedList::new();
            assert_eq!(None, list.pop_back(&mut token));

            list.push_back(1, &mut token);

            list.push_back(2, &mut token);
            list.push_back(3, &mut token);
            assert_eq!(Some(3), list.pop_back(&mut token));
            assert_eq!(Some(2), list.pop_back(&mut token));

            list.push_back(3, &mut token);
            assert_eq!(Some(3), list.pop_back(&mut token));

            assert_eq!(Some(1), list.pop_back(&mut token));
            assert_eq!(None, list.pop_back(&mut token));
        });
    }
}
