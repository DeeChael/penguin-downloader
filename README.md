# Penguin Downloader

一个基于 Kotlin 的音乐下载器，数据源可拓展，支持多种音质下载、歌词下载、元数据嵌入等功能。

## 功能特性

- 支持数据源扩展
- 支持搜索歌曲、专辑、歌单
- 支持多种音质下载（标准、高品质、无损、Hi-Res 等）
- 支持下载歌词
- 支持嵌入元数据（标题、艺术家、专辑、封面等）
- 支持账号登录（二维码扫描）
- 支持多线程下载
- 交互式命令行界面

## 模块结构

```
penguin-downloader/
├── build.gradle.kts              # 主程序构建配置
├── settings.gradle.kts           # 项目设置，包含所有模块
├── core/                         # 核心模块
│   └── src/main/kotlin/
│       ├── model/                # 数据模型（SongInfo, SongDetail 等）
│       ├── provider/             # Provider 接口定义
│       └── plugin/               # 插件系统实现
└── src/main/kotlin/              # 主程序入口
    └── com/penguin/downloader/
        ├── Main.kt               # 命令行入口
        └── download/             # 下载逻辑
```

## 安装

### 从源码构建

```bash
git clone https://github.com/DeeChael/penguin-downloader.git
cd penguin-downloader

# 构建
./gradlew build
```

构建完成后：
- `cli` 主程序位于 `cli/build/libs/penguin-downloader-cli-1.0.0.jar`
- （未完成）`tui` 主程序位于 `tui/build/libs/penguin-downloader-tui-1.0.0.jar`

## 使用方法

### 启动程序
直接运行就可以获取帮助信息
```bash
java -jar penguin-downloader-cli-1.0.0.jar
```

### 通过链接下载

直接传入音乐链接，程序会自动识别数据源和内容类型：  
（数据源支持即可，不方便展示）  

链接下载支持附加参数

```bash
# 指定输出目录
java -jar penguin-downloader-cli-1.0.0.jar "https://xxx.com/xxx" -o ~/Music

# 指定音质和下载歌词
java -jar penguin-downloader-cli-1.0.0.jar "https://xxx.com/xxx" -q 3 -l
```

### 登录

```bash
java -jar penguin-downloader-cli-1.0.0.jar -L <provider>
```

登录时会显示二维码，使用对应 App 扫描登录。登录成功后凭证会保存在 `credential.json` 文件中。

### 搜索并下载

所有操作都需要指定音乐源

搜索单曲
```bash
java -jar penguin-downloader-cli-1.0.0.jar -P <provider> -s 梦回还
```

搜索专辑
```bash
java -jar penguin-downloader-cli-1.0.0.jar -P <provider> -S hoyo-mix
```

### 下载歌单

```bash
# 列出用户歌单
java -jar penguin-downloader-cli-1.0.0.jar -P <provider> -r

# 通过歌单ID下载
java -jar penguin-downloader-cli-1.0.0.jar -P <provider> -p 123456

# 使用4线程下载歌单
java -jar penguin-downloader-cli-1.0.0.jar -P <provider> -p 123456 -t 4
```

### 下载专辑

```bash
java -jar penguin-downloader-cli-1.0.0.jar -P <provider> -a 82396066
```

### 下载单曲

```bash
# 通过歌曲ID下载
java -jar penguin-downloader-cli-1.0.0.jar -P <provider> -i 631519335
```

### 其他选项

```bash
# 强制重新下载（覆盖已存在的文件）
java -jar penguin-downloader-cli-1.0.0.jar -P <provider> -p 123456 --force

# 检查文件大小是否正确
java -jar penguin-downloader-cli-1.0.0.jar -P <provider> -p 123456 -c

# 同时下载歌词
java -jar penguin-downloader-cli-1.0.0.jar -P <provider> -s 梦回还 -l

# 指定输出目录
java -jar penguin-downloader-cli-1.0.0.jar -P <provider> -s 晴天 -o ~/Music

# 指定文件名格式
java -jar penguin-downloader-cli-1.0.0.jar -P <provider> -s hoyomix -f "{track} {title} - {artist}"
```

### 命令行参数

| 参数                     | 说明                                                   |
|------------------------|------------------------------------------------------|
| `--force`              | 强制重新下载，覆盖已存在的文件                                      |
| `--clean`              | 请除已经下载的文件                                            |
| `-L, --login`          | 登录指定数据源                       |
| `-P, --provider`       | 数据源，必选                        |
| `-a, --album`          | 通过专辑ID或MID下载                                         |
| `-c, --check`          | 检查已下载文件大小是否正确                                        |
| `-f, --format`         | 文件名格式: {track}, {title}, {artist}, {album}, {provider} |
| `-i, --id`             | 通过歌曲ID下载                                             |
| `-l, --lyrics`         | 同时下载歌词                                               |
| `-o, --output`         | 输出目录（默认为当前目录）                                        |
| `-p, --playlist`       | 通过歌单ID下载                                             |
| `-q, --quality`        | 音质等级（默认为最高可用音质）                                      |
| `-r, --list-playlists` | 列出用户歌单                                               |
| `-s, --search-song`    | 搜索并下载歌曲（交互式选择）                                       |
| `-S, --search-album`   | 搜索并下载专辑（交互式选择）                                       |
| `-t, --threads`        | 下载线程数（1-8，默认1）                                       |
| `-x, --logout`         | 登出指定数据源                                              |

### 文件名格式

| 占位符 | 说明     |
|--------|--------|
| `{track}` | 专辑音轨编号 |
| `{title}` | 歌曲标题   |
| `{artist}` | 艺术家    |
| `{album}` | 专辑名    |
| `{provider}` | 数据源名称  |

### 下载目录结构

```
<输出目录>/
├── songs/           # 单曲下载
├── albums/          # 专辑下载
│   └── <专辑名>/
└── playlists/       # 歌单下载
    └── <歌单名>/
```

## 插件系统

本项目支持通过插件扩展音乐数据源。将插件 JAR 文件放入 `plugins/` 目录，程序启动时会自动加载。

### 开发插件

1. 添加 `core` 模块为依赖
2. 创建实现 `MusicProvider` 接口的类
3. 创建 `META-INF/services/com.penguin.downloader.provider.MusicProvider` 文件，写入实现类全名
4. 打包为 JAR 文件

```kotlin
class MyProvider : MusicProvider {
    override val name: String = "myprovider"
    override val qualityLevels: Map<Int, String> = mapOf(
        1 to "标准音质",
        2 to "高品质",
        3 to "无损音质"
    )
    override val isLoggedIn: Boolean
        get() = // ...
    
    override suspend fun login(): Boolean { /* ... */ }
    override suspend fun searchSongs(keyword: String, page: Int, num: Int): SearchResult { /* ... */ }
    // 实现其他方法...
}
```

## 开源协议

本项目采用 MIT 协议开源，详见 [LICENSE](LICENSE) 文件。

## 免责声明

本项目仅供学习交流使用，请勿用于商业用途。使用本项目下载的音乐文件仅供个人试听，请在 24 小时内删除。支持正版，购买正版音乐是对音乐人最好的支持。

本项目仅提供下载框架，不提供任何音乐的下载数据源。
