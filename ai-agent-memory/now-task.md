# 现在的任务

解决以下问题

现有的bug：

- 添加任务后，对于当前任务状态的检测有问题，经常任务还在运行就输出“下载任务已全部完成”
- 在抓取收藏夹时看不到进度，只能看到毫无动静的等待直到直接输出抓取结果，
  - 对于抓取收藏夹，你可以使用`https://space.bilibili.com/234561771/favlist?fid=3685051971`这个地址用于测试，这个收藏夹中有888条记录，你可以以此测试进度显示，但不要实际下载，
- 需要启动bbdown时无反应，希望改成不是使用子进程启动，而是在windows中，以独立窗口启动，（此项无须在本次开发测试，开发环境是 wsl ubuntu，在所有任务完成然后报告用户之后，由用户自行测试效果）

需要改进的内容：

- 把要传给API的存储路径和需要程序检查的存储路径分离开，
  - 比如我当前的情况是bbdown服务运行在windows中，传给API的存储路径是`D:\download-buffer\BBDown\star\test-1`，
  - 但是实际上如果在wsl中直接运行，需要检测的路径应该是`/mnt/d/download-buffer/BBDown/star/test-1`

需要新增的内容：

- 能在程序的全局配置中设置默认的：
  - 下载目录
  - BBDown serve 地址
  - File Pattern
  - Multi File Pattern
- 在录入新收藏夹时，如果用户略过上述项，则使用全局默认配置（在输入时提示此默认值），

# 注意

在本次开发中，不需要使用dry-run测试，因为已经分离了bbdown，当前bbdown以服务器形式运行在windows中，本程序除了bbdown的其他部分已经均可以在wsl中实现正常运行，

我已经在`.config/bilibili_favlist_helper/config.json`中留下了名为`测试用`的一项配置，其中的`download_dir`值为在windows中的存储路径，`csv_path`为在wsl中对应的可访问文件路径，使用的服务器地址为`http://192.168.31.241:23333`

已设置不需要在开始补缺时启动bbdown，直接使用外部API服务，

你需要在本次开发中实际的测试和运行此`测试用`配置，

因为当文件已存在时，不会开始下载，所以你可以删除`/mnt/d/download-buffer/BBDown/star/test-1/`中的视频文件，以便重复测试，