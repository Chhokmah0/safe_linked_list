use crate::*;

#[test]
fn push_and_pop_back() {
    GhostToken::new(|token| {
        let mut list = SafeLinkedList::with_token(token);
        assert_eq!(None, list.pop_back());

        list.push_back(1);

        list.push_back(2);
        list.push_back(3);
        assert_eq!(Some(3), list.pop_back());
        assert_eq!(Some(2), list.pop_back());

        list.push_back(3);
        assert_eq!(Some(3), list.pop_back());

        assert_eq!(Some(1), list.pop_back());
        assert_eq!(None, list.pop_back());
    });
}

#[test]
fn push_and_pop_front() {
    GhostToken::new(|token| {
        let mut list = SafeLinkedList::with_token(token);
        assert_eq!(None, list.pop_front());

        list.push_front(1);

        list.push_front(2);
        list.push_front(3);
        assert_eq!(Some(3), list.pop_front());
        assert_eq!(Some(2), list.pop_front());

        list.push_front(3);
        assert_eq!(Some(3), list.pop_front());

        assert_eq!(Some(1), list.pop_front());
        assert_eq!(None, list.pop_front());
    });
}

#[test]
fn front_and_back() {
    GhostToken::new(|token| {
        let mut list = SafeLinkedList::with_token(token);
        assert_eq!(None, list.pop_front());

        list.push_front(1);
        list.push_front(2);
        list.push_front(3);

        assert_eq!(Some(1), list.pop_back());
        assert_eq!(Some(2), list.pop_back());
        assert_eq!(Some(3), list.pop_back());
        assert_eq!(None, list.pop_back());

        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        assert_eq!(Some(1), list.pop_front());
        assert_eq!(Some(2), list.pop_front());
        assert_eq!(Some(3), list.pop_front());
        assert_eq!(None, list.pop_front());
    });
}

#[test]
fn into_iter() {
    GhostToken::new(|token| {
        let mut list = SafeLinkedList::with_token(token);
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);
        let vec: Vec<_> = list.into_iter().collect();
        assert_eq!(vec, vec![1, 2, 3]);
    });
}

#[test]
fn multi_lists() {
    GhostToken::new(|token1| {
        GhostToken::new(move |token2| {
            let mut list1 = SafeLinkedList::with_token(token1);
            let mut list2 = SafeLinkedList::with_token(token2);
            list1.push_back(1);
            list1.push_back(2);
            list2.push_back(1);
            list2.push_back(2);
            for _ in list1.into_iter() {
                list2.push_back(3);
            }
            let vec: Vec<_> = list2.into_iter().collect();
            assert_eq!(vec, vec![1, 2, 3, 3]);
        })
    })
}

#[test]
fn iter() {
    GhostToken::new(|token| {
        let mut list = SafeLinkedList::with_token(token);
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);
        let vec: Vec<_> = list.iter().map(|x| *x).collect();
        let vec2: Vec<_> = list.into_iter().collect();
        assert_eq!(vec, vec2);
    })
}

#[test]
fn iter2() {
    GhostToken::new(|token| {
        let mut list = SafeLinkedList::with_token(token);
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);
        let mut cnt = 0;
        for _ in list.iter() {
            for _ in list.iter() {
                cnt += 1;
            }
        }
        assert_eq!(cnt, 9);

        let vec: Vec<_> = list.into_iter().collect();
        assert_eq!(vec, vec![1, 2, 3]);
    })
}

#[test]
fn drop() {
    GhostToken::new(|token| {
        let mut list = SafeLinkedList::with_token(token);
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);
    })
}
