# arxiv_bot

arxiv_bot可以自动抓取[arXiv](https://arxiv.org)上最新发表的文章信息，并将其发布到指定的社交媒体（目前只支持知乎想法）上。

## 特点

arxiv_bot同arXiv一样，主要基于学科（Subject）对文章进行划分。一个运行在服务器上的bot实例往往负责推送有关一个学科或其下的特定子学科
（Subject Area）的文章。而一篇文章可能会涉及多个领域，arxiv_bot允许一个实例通过某些方式与其他实例协调通信，并由负责该文章的主领域
（Primary Subject）的实例发布文章信息，其他实例进行转发操作（暂未实现）。

## 依赖
arxiv_bot现阶段只在Linux上进行完整功能测试，暂不考虑支持除*nix外的系统。

低于以下版本的依赖未经测试，但可能也可以顺利运行。
### 编译依赖
* Cargo ( >= 1.33.0 Nightly )
* make
* binutils
* pkg-config

### 运行依赖
* SQLite ( >= 3.24.0 )
* OpenSSL ( >= 1.1.0 )

## 部署 
请确保SQLite和OpenSSL的安装路径都在`LD_LIBRARY_PATH`中，如果使用Nix包管理器，可以直接运行`nix-shell arxiv_bot.nix`来安装库依赖
并启动一个包含该依赖的全新shell环境。

首先安装Rust ORM [Diesel](https://diesel.rs)的数据库部署/迁移工具`diesel_cli`，
用户也可以选择不使用`diesel_cli`并手动将`migrations`目录中的sql文件导入到指定数据库中。
```bash
PATH="$PATH:~/.cargo/bin"
cargo install diesel_cli --no-default-features --features "sqlite"
```

然后克隆代码库，指定一个SQLite数据库文件（如果不存在则创建一个）并初始化它。
```bash
git clone https://github.com/YuumuKonpaku/arxiv_bot.git
cd arxiv_bot
diesel setup --database-url='/path/to/db'
```

最后，编译代码库并将最终二进制文件放到合适位置即可。
```bash
cargo +nightly build --release
mv target/release/arxiv_bot /path/to/bin
```
如果选择将其放入`~/.cargo/bin`则可以直接运行`cargo +nightly install --path .`而无须手动复制编译结果。

## 配置
除动态链接库之外，arxiv_bot只需一个配置文件及SQLite数据库文件即可正常运行，不对存储路径有任何要求。任何可配置项均需通过配置文件
进行配置，所有的命令行参数都会被忽略。用户可以选择参考项目根目录下的`arxiv_bot.toml.example`进行配置。

### 命令格式
```bash
arxiv_bot # 在当前目录下寻找arxiv_bot.toml
arxiv_bot /path/to/conf # 或显式指定一个配置文件
```

### 配置文件格式
arxiv_bot使用[TOML](https://github.com/toml-lang/toml)作为配置文件格式
