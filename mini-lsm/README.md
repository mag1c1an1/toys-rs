get_with_ts 与 freeze_memtable_with_memtable 之间是没有并发问题的

Arc<RwLock<Arc<T>>> 配合 改clone的方式提高并发

因为`freeze_memtable_with_memtable` 直接clone了state，在这个克隆的state上修改最后即使写回到了state，也就是把之前旧的state的ref count -1，而读其实一直是读的原来的。
