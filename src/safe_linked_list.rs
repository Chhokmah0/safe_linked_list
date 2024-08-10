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

    pub fn is_empty(&self) -> bool {
        self.len == 0
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
        self.len -= 1;
        // GhostCell 无法移动，而 Full::into_inner 可以在不移动的情况下拿出内部所有权
        Some(Full::into_inner(Full::join(one, two)).into_inner().elem)
    }

    pub fn push_front(&mut self, elem: T, token: &mut GhostToken<'id>) {
        let (one, two) = Full::split::<1, 1>(Full::new(GhostCell::new(Node::with_elem(elem))));
        if let Some(right) = self.head.take() {
            right.borrow_mut(token).prev = Some(one);
            two.borrow_mut(token).next = Some(right);
            self.head = Some(two);
        } else {
            self.head = Some(one);
            self.tail = Some(two);
        }
        self.len += 1;
    }

    pub fn pop_front(&mut self, token: &mut GhostToken<'id>) -> Option<T> {
        let two = self.head.take()?;
        let one = if let Some(right) = two.borrow_mut(token).next.take() {
            let one = right.borrow_mut(token).prev.take().unwrap();
            self.head = Some(right);
            one
        } else {
            self.tail.take().unwrap()
        };
        self.len -= 1;
        // GhostCell 无法移动，而 Full::into_inner 可以在不移动的情况下拿出内部所有权
        Some(Full::into_inner(Full::join(one, two)).into_inner().elem)
    }
}

// IntoIter 存在的期间会占用整个 token，这是否合理呢？
// 应该是合理的，当有多个不相关数据结构时，应该创建多个 token 分别进行管理
pub struct IntoIter<'a, 'id, T> {
    token: &'a mut GhostToken<'id>,
    list: SafeLinkedList<'id, T>,
}

impl<'id, T> SafeLinkedList<'id, T> {
    pub fn into_iter<'a>(self, token: &'a mut GhostToken<'id>) -> IntoIter<'a, 'id, T> {
        IntoIter {
            token: token,
            list: self,
        }
    }
}

impl<'a, 'id, T> Iterator for IntoIter<'a, 'id, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.list.pop_front(self.token)
    }
}

pub struct Iter<'a, 'id, T> {
    token: &'a GhostToken<'id>,
    node_ptr: Option<&'a Half<'id, T>>,
}

impl<'id, T> SafeLinkedList<'id, T> {
    pub fn iter<'a, 'b: 'a>(&'a self, token: &'b GhostToken<'id>) -> Iter<'a, 'id, T> {
        Iter {
            token,
            node_ptr: self.head.as_ref(),
        }
    }
}

impl<'a, 'id, T> Iterator for Iter<'a, 'id, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(rc) = self.node_ptr.take() {
            self.node_ptr = rc.borrow(self.token).next.as_ref();
            Some(&rc.borrow(self.token).elem)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_and_pop_back() {
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

    #[test]
    fn push_and_pop_front() {
        GhostToken::new(|mut token| {
            let mut list = SafeLinkedList::new();
            assert_eq!(None, list.pop_front(&mut token));

            list.push_front(1, &mut token);

            list.push_front(2, &mut token);
            list.push_front(3, &mut token);
            assert_eq!(Some(3), list.pop_front(&mut token));
            assert_eq!(Some(2), list.pop_front(&mut token));

            list.push_front(3, &mut token);
            assert_eq!(Some(3), list.pop_front(&mut token));

            assert_eq!(Some(1), list.pop_front(&mut token));
            assert_eq!(None, list.pop_front(&mut token));
        });
    }

    #[test]
    fn front_and_back() {
        GhostToken::new(|mut token| {
            let mut list = SafeLinkedList::new();
            assert_eq!(None, list.pop_front(&mut token));

            list.push_front(1, &mut token);
            list.push_front(2, &mut token);
            list.push_front(3, &mut token);

            assert_eq!(Some(1), list.pop_back(&mut token));
            assert_eq!(Some(2), list.pop_back(&mut token));
            assert_eq!(Some(3), list.pop_back(&mut token));
            assert_eq!(None, list.pop_back(&mut token));

            list.push_back(1, &mut token);
            list.push_back(2, &mut token);
            list.push_back(3, &mut token);

            assert_eq!(Some(1), list.pop_front(&mut token));
            assert_eq!(Some(2), list.pop_front(&mut token));
            assert_eq!(Some(3), list.pop_front(&mut token));
            assert_eq!(None, list.pop_front(&mut token));
        });
    }

    #[test]
    fn into_iter() {
        GhostToken::new(|mut token| {
            let mut list = SafeLinkedList::new();
            list.push_back(1, &mut token);
            list.push_back(2, &mut token);
            list.push_back(3, &mut token);
            let vec: Vec<_> = list.into_iter(&mut token).collect();
            assert_eq!(vec, vec![1, 2, 3]);
        });
    }

    #[test]
    fn multi_lists() {
        GhostToken::new(|mut token1| {
            GhostToken::new(move |mut token2| {
                let mut list1 = SafeLinkedList::new();
                let mut list2 = SafeLinkedList::new();
                list1.push_back(1, &mut token1);
                list1.push_back(2, &mut token1);
                list2.push_back(1, &mut token2);
                list2.push_back(2, &mut token2);
                for _ in list1.into_iter(&mut token1) {
                    list2.push_back(3, &mut token2);
                }
                let vec: Vec<_> = list2.into_iter(&mut token2).collect();
                assert_eq!(vec, vec![1, 2, 3, 3]);
            })
        })
    }

    #[test]
    fn iter() {
        GhostToken::new(|mut token| {
            let mut list = SafeLinkedList::new();
            list.push_back(1, &mut token);
            list.push_back(2, &mut token);
            list.push_back(3, &mut token);
            let vec: Vec<_> = list.iter(&token).map(|x| *x).collect();
            let vec2: Vec<_> = list.into_iter(&mut token).collect();
            assert_eq!(vec, vec2);
        })
    }

    #[test]
    fn iter2() {
        GhostToken::new(|mut token| {
            let mut list = SafeLinkedList::new();
            list.push_back(1, &mut token);
            list.push_back(2, &mut token);
            list.push_back(3, &mut token);
            let mut cnt = 0;
            for _ in list.iter(&token) {
                for _ in list.iter(&token) {
                    cnt += 1;
                }
            }
            assert_eq!(cnt, 9);
            list.pop_back(&mut token);
            list.pop_back(&mut token);
            list.pop_back(&mut token);
        })
    }
}
