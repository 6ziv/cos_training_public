## 第一次实验



#### 实验目标

`a1`：通过 `mmu_enable` 与`mmu_disable`功能控制页表机制的开关。

`a2`：通过`sv39` 与 `sv48` 功能控制具体实现的机制。



#### 实验内容

###### 1. 阅读`mmu_identity`的实现

`mmu_identity` 的实现由三部分组成。在`pre_mmu`部分，建立三个`gigapage`，分别位于虚地址空间的`0x80000000..0xc0000000`，`0xffffffc080000000..0xffffffc0c0000000`，以及`0xffffffffc0000000..0xffffffffffffffff`三处。它们指向的物理地址同样都是`0x80000000..0xc0000000`。

`enable_mmu`设置好相应的`satp`寄存器之后，使用`sfence_vma_all`来保证程序对页表的读写不会跨过这一语句、且页表缓存应当失效。

最后，`post_mmu`修正程序的栈指针与返回地址，从而使得程序能够正常向下执行。这里在二者上统一加上了`PHYS_VIRT_OFFSET`，也就使得`0x80000000..0xc0000000`中的位置会被替换为`0xffffffc080000000..0xffffffc0c0000000`中的对应位置。因为栈指针此时指向`boot_stack`中某处、`ra`则在`lib.rs`中 `_start`函数调用`post_mmu`处，二者都在内核镜像中，所以都位于这段区间内，因此可以正常执行。



###### 2. 实现`mmu_alterable`

利用`#[cfg(feature = "enable")]` 和`#[cfg(feature = "disable")]`进行条件编译。

要注意的是这里面的`feature`名字是`enable`/`disable` 而非 `Makefile` 传入的`mmu_enable`/`mmu_disable`，这是因为`libos/Cargo.toml`在声明`feature`时进行了重命名。

在`feature = "enable"`的情况下，只需要照搬`mmu_identity`的实现即可。

在`feature = "disable"`的情况下，`pre_mmu`和`post_mmu`都可以使用空实现，而`enable_mmu`则把`satp`设置为`Bare`模式。

特别地，`KERNEL_BASE` 被外部代码引用了，因此即使在`mmu_disable`时，也不能删掉这一常量的定义。



###### 3. 实现`mmu_scheme`

利用`#[cfg(feature = "sv39")]` 和`#[cfg(feature = "sv48")]`进行条件编译。

对于前者可以直接采用`mmu_identity`的实现。因此我们只需要对`sv48`实现`pre_mmu`，`enable_mmu`和`post_mmu`即可。

其中，`post_mmu`可以保持不变、`enable_mmu`只需要调整`satp`的模式即可。因此重点将在于`pre_mmu`中对页表的初始化。

因为`sv48`模式下，`1GiB`的`gigapage`并非最大的，也就是说不在页表的第一层。因此我们需要建立一个分层的页表。

对于`0x80000000..0xc0000000`，它位于第一个`terapage`上，因此我们在主页表下标为`0`的位置写入`((root_page_id+1) << 10) | 0x01`。这意味着，它指向主页表之后的下一个页表，并且只有`V`标志位被设置。

对于另外两段地址，它们位于最后一个`terapage`上，所以我们和刚才类似地，把主页表上下标为`0x1ff`的页表项指向主页表之后的第二个页表，并且只设置`V`标志位。

可以看到，我们用了连续的三个页表，一共有`0x600`个页表项的大小，因此我们将`BOOT_PT_SV48`改为长为`0x600`的数组。

当然，这三个页表并不一定要是连续的。但是如果在其它地方声明代表另外两段页表的数组的话，除非修改`linker_riscv64.lds`文件，否则不一定保证它们是以`4KiB`边界对齐的。所以把它们直接与主页表合起来，可以省下不少麻烦。



建立好主页表之后，只要仿照`sv39`，把第一个页表的下标`2`处、第二个页表的下标`0x102`和`0x1FF`处填入`(0x80000 << 10) | 0xef`即可。



#### 遇到的问题

* 最开始不知道怎么设置非叶子页表项的GADUXWRV，直接设置成了`GAD____V`，结果程序无法正常运行。

* 经常写错地址。后来把需要的虚地址拆分成九位之后记下来才解决。
