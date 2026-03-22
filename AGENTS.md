# Gelbooru 图片爬虫项目

## 项目概述

这是一个基于 **Spring Boot 2.7.0** 开发的 Gelbooru 图片爬虫应用，用于自动化下载 Gelbooru 网站的图片资源。

### 主要技术栈

- **框架**: Spring Boot 2.7.0
- **Java 版本**: JDK 1.8
- **HTTP 客户端**: OkHttp3
- **HTML 解析**: Jsoup
- **工具库**: 
  - Hutool 5.8.3
  - Apache Commons (Lang3, Text, Collections4, IO, Codec)
  - Google Guava
  - Gson
  - Lombok

### 项目架构

```
com.gelbooru
├── Main.java                    # 应用入口
├── configuration/               # 配置类
│   ├── GelbooruProperties.java  # Gelbooru 站点配置
│   ├── HttpProperties.java      # HTTP 请求配置
│   ├── HttpConfiguration.java   # HTTP 客户端配置
│   └── HttpProxyProperties.java # 代理配置
├── context/                     # 数据模型/上下文
│   ├── GelbooruPage.java        # 分页查询上下文
│   ├── GelbooruPost.java        # 帖子/图片信息
│   ├── GelbooruPostStatistics.java # 帖子统计信息
│   ├── GelbooruTag.java         # 标签信息
│   └── OssRefBean.java          # 存储引用
├── controller/                  # 控制器层
├── enums/                       # 枚举定义
├── http/                        # HTTP 相关
│   ├── HttpSupport.java         # HTTP 请求支持
│   ├── RequestFactory.java      # 请求工厂
│   ├── CookieJarRepository.java # Cookie 仓库接口
│   └── CookieJarFileRepository.java # 文件 Cookie 存储
├── job/                         # 定时任务
│   └── GelbooruServiceJob.java  # 爬虫任务
└── service/                     # 服务层
    ├── GelbooruService.java     # 爬虫服务接口
    ├── OssService.java          # 存储服务接口
    └── impl/
        ├── GelbooruServiceImpl.java  # 爬虫服务实现
        └── LocalOssServiceImpl.java  # 本地存储实现
```

## 核心功能

### 1. 图片抓取
- 支持按标签搜索 Gelbooru 图片
- 自动过滤视频和动画 (`video`, `animated` 标签排除)
- 优先抓取高分辨率图片 (`highres` 标签)
- 每页 42 张图片，支持分页遍历

### 2. 智能分类存储
下载的图片按以下结构自动分类存储：
```
D:/project/gelbooru/imgs/
└── {postedDate}/                    # 发布日期
    └── {rating}/                     # 内容分级 (safe/questionable/explicit)
        └── {copyright}/               # 版权/作品名
            └── [{character}]{id}({artist}).{ext}  # 文件名
```

### 3. 元数据提取
- 提取标签信息（艺术家、角色、版权、一般标签）
- 获取图片统计信息（尺寸、评分、发布时间、来源、得分）
- 解析原图链接

## 构建与运行

### 环境要求
- JDK 1.8+
- Maven 3.6+

### 构建命令

```bash
# 编译打包
mvn clean package

# 跳过测试打包
mvn clean package -DskipTests
```

### 运行命令

```bash
# 直接运行
java -jar target/gelbooru.jar

# 或者使用 Maven
mvn spring-boot:run
```

应用默认在 **8082** 端口启动。

### 配置说明

配置文件位置: `src/main/resources/application.properties`

```properties
# 服务端口
server.port=8082

# HTTP User-Agent
http.userAgent=Mozilla/5.0 (Windows NT 10.0; Win64; x64)...

# Cookie 存储目录
http.cookieDir=/project/gelbooru/cookie
```

## 开发规范

### 代码风格
- 使用 Lombok 注解简化 POJO 代码 (`@Data`, `@RequiredArgsConstructor`, `@Slf4j`)
- 使用 `final` 关键字定义常量
- 采用流式 API (Stream API) 处理集合

### 日志规范
- 使用 Lombok 的 `@Slf4j` 注解
- Debug 级别日志需判断 `log.isDebugEnabled()`
- 错误日志需记录异常堆栈: `log.error(msg, e)`

### 异常处理
- 使用 `@SneakyThrows` 简化受检异常处理
- HTTP 请求异常需捕获并记录，避免中断流程

### 文件存储规范
- 文件名特殊字符自动替换为下划线 (`:`, ` `, `<`, `>`, `__`)
- 使用 `StringUtils.replaceEachRepeatedly` 确保多次替换

## 使用示例

### 运行爬虫任务

```java
// 注入任务类
@Autowired
private GelbooruServiceJob job;

// 执行搜索标签的抓取任务
job.execute("tag_name");
```

任务会：
1. 从第 1 页开始遍历
2. 获取每页 42 张图片的缩略图
3. 访问每张图片详情页获取元数据
4. 下载两天内发布的图片
5. 按分类结构保存到本地
6. 自动跳过已存在的图片

## 注意事项

1. **存储路径**: 默认存储在 `D:/project/gelbooru/imgs/`，确保该目录存在或有写入权限
2. **Cookie 持久化**: Cookie 自动保存到 `cookie/` 目录，支持会话保持
3. **请求频率**: 代码中未显式限制请求频率，请根据 Gelbooru 的 robots.txt 和实际使用合理控制
4. **代理支持**: 可通过 `HttpProxyProperties` 配置代理服务器

## 测试

测试类位于 `src/test/java/com/gelbooru/service/`：

- `GelbooruServiceTest.java` - 服务层测试
- `GelbooruCharacterJobTest.java` - 角色抓取任务测试
- `GelbooruCopyRightJobTest.java` - 版权抓取任务测试

运行测试：

```bash
mvn test
```

## 依赖管理

项目使用 Maven 进行依赖管理，主要依赖版本在 `pom.xml` 中定义：
- Spring Boot: 2.7.0
- Hutool: 5.8.3
- Lombok: 1.18.24
- Jsoup: 1.15.3
- Gson: 2.10.1
- Guava: 32.0.0-android

---
*最后更新: 2026-03-22*
