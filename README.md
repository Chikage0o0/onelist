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
3. 在"重定向"中选择"Web"，填写`http://localhost:8080/auth/callback`。
4. 注册后，点击API权限，添加"Files.Read.All"和"offline_access"权限。
5. 点击"证书和密码"，新建客户端密码，保存好客户端密码。
6. 返回概述，复制应用程序(客户端) ID。

### 配置
创建`config.toml`文件，填写以下内容：
```toml
[auth]
client_id = "应用程序(客户端) ID"
client_secret = "客户端密码"


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

