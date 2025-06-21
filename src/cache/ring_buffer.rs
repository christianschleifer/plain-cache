#[derive(Debug)]
pub(crate) struct RingBuffer<T> {
    head: usize,
    len: usize,
    buffer: Vec<Option<T>>,
}

impl<T> RingBuffer<T> {
    pub(crate) fn with_capacity(capacity: usize) -> RingBuffer<T> {
        let mut buffer = Vec::with_capacity(capacity);
        buffer.resize_with(capacity, || None);
        RingBuffer {
            head: 0,
            len: 0,
            buffer,
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub(crate) fn is_full(&self) -> bool {
        self.len == self.buffer.capacity()
    }

    pub(crate) fn get(&self, index: usize) -> Option<&T> {
        self.buffer.get(index).and_then(Option::as_ref)
    }

    /// Adds an item to the back of the queue.
    pub(crate) fn push_back(&mut self, value: T) -> Option<usize> {
        if self.is_full() {
            return None;
        }

        // buffer.cap   - - - - -
        // buffer.len   - - - - -
        // len                - -
        // head               |
        //             [N N N S S ]
        let physical_idx = self.wrap_add(self.head, self.len);
        self.buffer[physical_idx] = Some(value);
        self.len += 1;
        // buffer.cap   - - - - -
        // buffer.len   - - - - -
        // len          -     - -
        // head               |
        //             [S N N S S ]
        Some(physical_idx)
    }

    /// Pops an element from the front of the queue and returns it.
    ///
    /// If the queue is empty, [None] is returned.
    pub(crate) fn pop_front(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            // buffer.cap   - - - - -
            // buffer.len   - - - - -
            // len          -     - -
            // head               |
            //             [S N N S S ]
            while self.len >= 1 {
                let t = self.buffer[self.head].take();

                self.head = self.wrap_add(self.head, 1);
                self.len -= 1;

                match t {
                    None => continue,
                    item @ Some(_) => {
                        return item;
                    }
                }
            }
            None
        }
    }

    /// Removes an element from the queue. Note that this will not immediately increase the len of
    /// the queue. Only calling using [RingBuffer::pop_front] will do this.
    ///
    /// ## Panics
    /// This method doesn't do an index check. Out of bound accesses will panic.
    pub(crate) fn remove(&mut self, index: usize) -> Option<T> {
        self.buffer[index].take()
    }

    fn wrap_add(&self, idx: usize, addend: usize) -> usize {
        let capacity = self.buffer.capacity();
        let idx = idx.wrapping_add(addend);
        if idx >= capacity { idx - capacity } else { idx }
    }
}

#[cfg(test)]
mod tests {
    use crate::cache::ring_buffer::RingBuffer;

    #[test]
    fn it_is_empty() {
        // given
        let mut ring_buffer = RingBuffer::with_capacity(1);

        // when
        ring_buffer.push_back(String::from("hello world")).unwrap();
        ring_buffer.pop_front();
        let is_empty = ring_buffer.is_empty();

        // then
        assert!(is_empty)
    }

    #[test]
    fn it_is_not_empty() {
        // given
        let mut ring_buffer = RingBuffer::with_capacity(1);

        // when
        ring_buffer.push_back(String::from("hello world")).unwrap();
        let is_empty = ring_buffer.is_empty();

        // then
        assert!(!is_empty)
    }

    #[test]
    fn it_is_full() {
        // given
        let mut ring_buffer = RingBuffer::with_capacity(1);

        // when
        ring_buffer.push_back(String::from("hello world")).unwrap();
        let is_full = ring_buffer.is_full();

        // then
        assert!(is_full)
    }

    #[test]
    fn it_is_not_full() {
        // given
        let mut ring_buffer = RingBuffer::with_capacity(1);

        // when
        ring_buffer.push_back(String::from("hello world")).unwrap();
        ring_buffer.pop_front();
        let is_full = ring_buffer.is_full();

        // then
        assert!(!is_full)
    }

    #[test]
    fn it_pushes_back_by_wrapping_around() {
        // given
        let mut ring_buffer = RingBuffer::with_capacity(5);
        ring_buffer.push_back(String::from("first")).unwrap();
        ring_buffer.push_back(String::from("second")).unwrap();
        ring_buffer.push_back(String::from("third")).unwrap();
        ring_buffer.push_back(String::from("fourth")).unwrap();
        ring_buffer.push_back(String::from("fifth")).unwrap();
        ring_buffer.pop_front();
        ring_buffer.pop_front();
        ring_buffer.pop_front();

        // buffer.cap   - - - - -
        // buffer.len   - - - - -
        // len                - -
        // head               |
        //             [N N N S S ]
        assert_eq!(ring_buffer.head, 3);
        assert_eq!(ring_buffer.buffer.len(), 5);
        assert_eq!(ring_buffer.len, 2);

        // when
        let idx = ring_buffer.push_back(String::from("sixth")).unwrap();

        // then

        // buffer.cap   - - - - -
        // buffer.len   - - - - -
        // len          -     - -
        // head               |
        //             [S N N S S ]
        assert_eq!(idx, 0);
        assert_eq!(ring_buffer.head, 3);
        assert_eq!(ring_buffer.buffer.len(), 5);
        assert_eq!(ring_buffer.len, 3);
    }

    #[test]
    fn it_pushes_back_when_fully_wrapped() {
        // given
        let mut ring_buffer = RingBuffer::with_capacity(5);
        ring_buffer.push_back(String::from("first")).unwrap();
        ring_buffer.push_back(String::from("second")).unwrap();
        ring_buffer.push_back(String::from("third")).unwrap();
        ring_buffer.push_back(String::from("fourth")).unwrap();
        ring_buffer.push_back(String::from("fifth")).unwrap();
        ring_buffer.pop_front();
        ring_buffer.pop_front();
        ring_buffer.pop_front();
        ring_buffer.pop_front();
        ring_buffer.pop_front();

        // buffer.cap   - - - - -
        // buffer.len   - - - - -
        // len
        // head         |
        //             [N N N N N ]
        assert_eq!(ring_buffer.head, 0);
        assert_eq!(ring_buffer.buffer.len(), 5);
        assert_eq!(ring_buffer.len, 0);

        // when
        let idx = ring_buffer.push_back(String::from("sixth")).unwrap();

        // then

        // buffer.cap   - - - - -
        // buffer.len   - - - - -
        // len          -
        // head         |
        //             [S N N N N ]
        assert_eq!(idx, 0);
        assert_eq!(ring_buffer.head, 0);
        assert_eq!(ring_buffer.buffer.len(), 5);
        assert_eq!(ring_buffer.len, 1);
    }

    #[test]
    fn it_does_not_push_back_when_full() {
        // given
        let mut ring_buffer = RingBuffer::with_capacity(5);
        ring_buffer.push_back(String::from("first")).unwrap();
        ring_buffer.push_back(String::from("second")).unwrap();
        ring_buffer.push_back(String::from("third")).unwrap();
        ring_buffer.push_back(String::from("fourth")).unwrap();
        ring_buffer.push_back(String::from("fifth")).unwrap();

        // when
        let option = ring_buffer.push_back(String::from("sixth"));

        // then
        assert!(option.is_none());
    }

    #[test]
    fn it_handles_deletions() {
        // given
        let mut ring_buffer = RingBuffer::with_capacity(5);
        ring_buffer.push_back(String::from("first")).unwrap();
        ring_buffer.push_back(String::from("second")).unwrap();
        ring_buffer.push_back(String::from("third")).unwrap();
        ring_buffer.push_back(String::from("fourth")).unwrap();
        ring_buffer.push_back(String::from("fifth")).unwrap();
        ring_buffer.pop_front();
        ring_buffer.pop_front();
        ring_buffer.pop_front();

        ring_buffer.remove(3);
        // buffer.cap   - - - - -
        // buffer.len   - - - - -
        // len                - -
        // head               |
        //             [N N N N S ]
        assert_eq!(ring_buffer.head, 3);
        assert_eq!(ring_buffer.buffer.len(), 5);
        assert_eq!(ring_buffer.len, 2);

        // when
        let item = ring_buffer.pop_front().unwrap();

        // then

        // buffer.cap   - - - - -
        // buffer.len   - - - - -
        // len
        // head         |
        //             [N N N N N ]
        assert!(ring_buffer.is_empty());
        assert_eq!(ring_buffer.head, 0);
        assert_eq!(ring_buffer.buffer.len(), 5);
        assert_eq!(ring_buffer.len, 0);
        assert_eq!(item, "fifth")
    }
}
