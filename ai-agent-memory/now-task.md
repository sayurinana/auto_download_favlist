# 现在的任务

对crates下的模块进行优化，具体期望如下（对应的需求，如果可以的话尽可能选用合适的现成crate）：

- 使用`console`或其他合适的crate来美化界面的输出和色彩等界面设计，

- 在抓取收藏夹时显示进度条（显示当前已获取数和总视频数而不是最大100%的百分比），

- 在检查缺漏时，不再每通过每个视频都直接调用bbdown来获取：

  - 在选择此项，进入这个环节时，先在一个子进程中启动`BBDown serve`，本机的服务地址是http://localhost:23333（此地址可通过配置指定，默认为此值），
    - 由于linux环境没有部署BBDown，无法启动，所以在测试时以dry-run行为代替，并不实际启动bbdown，我会在网络环境中部署好，启动好`bbdown serve`，服务地址为http://192.168.31.241:23333，这是一个有效的地址，
  - 对于找到的缺漏项，需要下载的视频，通过向API（API信息详见[BBDown/json-api-doc.md at master · nilaoda/BBDown](https://github.com/nilaoda/BBDown/blob/master/json-api-doc.md)）发起http请求来添加下载任务，而不是直接调用`bbdown bvid……`，
  - 把所有的下载任务添加完后，每500毫秒（此时间可通过配置指定，默认为此值）获取一次任务状态，在当前运行的任务为空时即为下载完成，关闭刚刚启动的bbdown进程（dry-run则无动作），向用户报告，然后再次检查缺漏项，但此次仅检查而不再次调用api，然后向用户报告检查结果，
  - 完成

  

