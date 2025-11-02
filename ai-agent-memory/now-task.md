# 现在的任务

我把旧的py程序源代码从src移到了py-src，

你需要先分析ai-agent-output中两个之前的任务需求optimized-task.md，

然后完成新任务：

- 使用rust实现两个程序
  - 其中auto_download_favlist改名为get_bilibili_favlist_bvid_list
- 先实现get_bilibili_favlist_bvid_list再实现billibili_favlist_download_helper
- 新的billibili_favlist_download_helper使用crate的形式调用get_bilibili_favlist_bvid_list的功能而不是通过shell调用二进制程序
- 只允许用户使用*方向键或WASD，回车键或空格，选中退出项或按下Esc*，来操作功能选项，而不是通过简述功能选项序号来选择，

