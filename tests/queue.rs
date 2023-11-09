use sudoku_game::queue::Queue;

#[test]
fn test_queue() {
    let mut queue: Queue<u32, 4> = Queue::empty();
    queue.push(1);
    queue.push(2);
    assert!(!queue.is_empty());
    queue.push(3);
    assert_eq!(queue.pop(), Some(1));
    queue.push(4);
    assert_eq!(queue.pop(), Some(2));
    queue.push(5);
    assert_eq!(queue.pop(), Some(3));
    assert_eq!(queue.pop(), Some(4));
    assert_eq!(queue.pop(), Some(5));
    assert_eq!(queue.pop(), None);
    assert!(queue.is_empty());
}

#[test]
#[should_panic]
fn test_queue_overflow() {
    let mut queue: Queue<u32, 4> = Queue::empty();
    queue.push(1);
    queue.push(2);
    queue.push(3);
    queue.push(4);
}
