pub(crate) struct RingBuffer<T> {
    head: usize,
    len: usize,
    buffer: Vec<Option<T>>,
}

impl<T> RingBuffer<T> {
    pub(crate) fn with_capacity(capacity: usize) -> RingBuffer<T> {
        RingBuffer {
            head: 0,
            len: 0,
            buffer: Vec::with_capacity(capacity),
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

    pub(crate) fn push_back(&mut self, value: T) -> Option<usize> {
        if self.is_full() {
            return None;
        }

        // buffer.cap   - - - - -
        // buffer.len   - - -
        // len            - -
        // head           |
        //             [N S S _ _ ]
        if self.buffer.len() < self.buffer.capacity() {
            self.len += 1;
            self.buffer.push(Some(value));
            // buffer.cap   - - - - -
            // buffer.len   - - - -
            // len            - - -
            // head           |
            //             [N S S S _ ]
            return Some(self.buffer.len() - 1);
        };

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

    pub(crate) fn pop_front(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            // buffer.cap   - - - - -
            // buffer.len   - - - - -
            // len          -     - -
            // head               |
            //             [N N N S S ]
            let t = self.buffer[self.head]
                .take()
                .expect("deletions not supported");
            self.head = self.wrap_add(self.head, 1);
            self.len -= 1;
            Some(t)
        }
    }

    fn wrap_add(&self, idx: usize, addend: usize) -> usize {
        self.wrap_index(idx.wrapping_add(addend), self.buffer.capacity())
    }

    fn wrap_index(&self, logical_index: usize, capacity: usize) -> usize {
        if logical_index >= capacity {
            logical_index - capacity
        } else {
            logical_index
        }
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
    fn it_pushes_back_by_pushing_to_buffer() {
        // given
        let mut ring_buffer = RingBuffer::with_capacity(5);
        ring_buffer.push_back(String::from("first")).unwrap();
        ring_buffer.push_back(String::from("second")).unwrap();
        ring_buffer.push_back(String::from("third")).unwrap();
        ring_buffer.pop_front();

        // buffer.cap   - - - - -
        // buffer.len   - - -
        // len            - -
        // head           |
        //             [N S S _ _ ]
        assert_eq!(ring_buffer.head, 1);
        assert_eq!(ring_buffer.buffer.len(), 3);
        assert_eq!(ring_buffer.len, 2);

        // when
        let idx = ring_buffer.push_back(String::from("fourth")).unwrap();

        // then

        // buffer.cap   - - - - -
        // buffer.len   - - - -
        // len            - - -
        // head           |
        //             [N S S S _ ]
        assert_eq!(idx, 3);
        assert_eq!(ring_buffer.head, 1);
        assert_eq!(ring_buffer.buffer.len(), 4);
        assert_eq!(ring_buffer.len, 3);
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
}
