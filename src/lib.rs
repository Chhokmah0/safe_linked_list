use ghost_cell::{GhostCell, GhostToken};
use static_rc::StaticRc;

pub struct SafeLinkedList<'id, T> {
    head: Option<Half<'id, T>>,
    tail: Option<Half<'id, T>>,
    len: usize,
    // 为了可以安全 drop，token 必须放在结构体内
    // 这导致一个 token 内只能创建一个 LinkedList 结构
    // 不过如果给 token 加上读写锁，或许可以比较方便地多线程？
    token: GhostToken<'id>,
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
    pub fn with_token(token: GhostToken<'id>) -> Self {
        Self {
            head: None,
            tail: None,
            len: 0,
            token,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn push_back(&mut self, elem: T) {
        let (one, two) = Full::split::<1, 1>(Full::new(GhostCell::new(Node::with_elem(elem))));
        if let Some(left) = self.tail.take() {
            left.borrow_mut(&mut self.token).next = Some(one);
            two.borrow_mut(&mut self.token).prev = Some(left);
            self.tail = Some(two);
        } else {
            self.head = Some(one);
            self.tail = Some(two);
        }
        self.len += 1;
    }

    pub fn pop_back(&mut self) -> Option<T> {
        let two = self.tail.take()?;
        let one = if let Some(left) = two.borrow_mut(&mut self.token).prev.take() {
            let one = left.borrow_mut(&mut self.token).next.take().unwrap();
            self.tail = Some(left);
            one
        } else {
            self.head.take().unwrap()
        };
        self.len -= 1;
        // GhostCell 无法移动，而 Full::into_inner 可以在不移动的情况下拿出内部所有权
        Some(Full::into_inner(Full::join(one, two)).into_inner().elem)
    }

    pub fn push_front(&mut self, elem: T) {
        let (one, two) = Full::split::<1, 1>(Full::new(GhostCell::new(Node::with_elem(elem))));
        if let Some(right) = self.head.take() {
            right.borrow_mut(&mut self.token).prev = Some(one);
            two.borrow_mut(&mut self.token).next = Some(right);
            self.head = Some(two);
        } else {
            self.head = Some(one);
            self.tail = Some(two);
        }
        self.len += 1;
    }

    pub fn pop_front(&mut self) -> Option<T> {
        let two = self.head.take()?;
        let one = if let Some(right) = two.borrow_mut(&mut self.token).next.take() {
            let one = right.borrow_mut(&mut self.token).prev.take().unwrap();
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

impl<'id, T> Drop for SafeLinkedList<'id, T> {
    fn drop(&mut self) {
        while !self.is_empty() {
            self.pop_front();
        }
    }
}

// IntoIter 存在的期间会占用整个 token，这是否合理呢？
// 应该是合理的，当有多个不相关数据结构时，应该创建多个 token 分别进行管理
pub struct IntoIter<'id, T> {
    list: SafeLinkedList<'id, T>,
}

impl<'id, T> SafeLinkedList<'id, T> {
    pub fn into_iter(self) -> IntoIter<'id, T> {
        IntoIter { list: self }
    }
}

impl<'id, T> Iterator for IntoIter<'id, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.list.pop_front()
    }
}

pub struct Iter<'a, 'id, T> {
    token: &'a GhostToken<'id>,
    node_ptr: Option<&'a Half<'id, T>>,
}

impl<'id, T> SafeLinkedList<'id, T> {
    pub fn iter<'a>(&'a self) -> Iter<'a, 'id, T> {
        Iter {
            token: &self.token,
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
mod test;
