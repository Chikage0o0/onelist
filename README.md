# Onelist
一个简单的OneDrive文件列表程序，使用Rust+Axum实现。

## 特性
- [x] 列出文件列表
- [x] 视频播放
- [x] 目录、文件信息缓存
- [x] 代理下载

## 使用
### 创建应用
1. 访问[Entra 管理中心](https://entra.microsoft.com/#view/Microsoft_AAD_RegisteredApps/ApplicationsListBlade/quickStartType~/null/sourceType/Microsoft_AAD_IAM)。
2. 点击"新注册"，填写应用名称，选择"帐户类型"为"任何组织目录中的帐户"。
3. 在"重定向"中选择"Web"，填写`http://localhost:10080/redirect`。
4. 注册后，点击API权限，添加"Files.Read.All"和"offline_access"权限。
5. 点击"证书和密码"，新建客户端密码，保存好客户端密码。
6. 返回概述，复制应用程序(客户端) ID。

### 配置
创建`config.toml`文件，填写以下内容：
```toml
[auth]
client_id = "应用程序(客户端) ID"
client_secret = "客户端密码"
# API 类型
# consumers 纯个人版
# organizations 纯组织
# common 个人版和组织版
type = "consumers"


[setting]
# 显示的OneDrive路径
home_dir = "/"
# 是否使用代理下载
use_proxy = false
# 展示的名称
name = "OneList"
# 开放的端口
port = 3000
```

### 本地连接与测试

建议首先在本地进行授权测试，以确保配置正确以及获取refresh_token。  
1. 访问`https://github.com/Chikage0o0/onelist/actions/workflows/build.yml`，下载最新的构建文件。
2. 将`config.toml`和`onelist`放在同一目录下。
3. 运行`./onelist`，根据命令行提示访问`https://login.microsoftonline.com`进行授权。
4. 授权成功后，将跳转的链接输入到命令行中，并回车。
5. 当提示`Starting the web server`时，访问`http://localhost:3000`，即可查看效果。
6. 确定效果正确后，可以使用`Ctrl+C`关闭程序，refreshtoken会保存在`config.toml`中，下次启动时会自动读取，无需再次授权。

### 部署
这里推荐使用Docker部署
```bash
docker run -d --name onelist -p 3000:3000 -v /path/to/config.toml:/app/config.toml chikage0o0/onelist
```