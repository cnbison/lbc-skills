---
name: mofa-fm-api
description: "MoFA FM Podcast Platform API client — browse shows/episodes, search podcasts, manage creator content, trending topics. Triggers: mofa.fm, 播客, podcast, fm api, 热搜"
requires_bins: python3
octos_only: true
always: false
---

# MoFA FM API

MoFA FM 播客平台 API 客户端，用于浏览节目、搜索内容、管理创作者内容、获取热搜数据。

## Base URL

```
https://mofa.fm/api/
```

## 认证方式

- **公开接口**: 无需认证
- **私有接口**: JWT Bearer Token
  ```
  Authorization: Bearer <access_token>
  ```

### 获取 Token

```bash
# 注册
POST /api/auth/register/
Body: {"username": "...", "email": "...", "password": "...", "password2": "..."}

# 登录
POST /api/auth/login/
Body: {"username": "...", "password": "..."}
Response: {"access": "...", "refresh": "..."}
```

## API 分类

### 1. 认证 API (`/api/auth/`)

| 端点 | 方法 | 说明 | 认证 |
|------|------|------|------|
| `/api/auth/register/` | POST | 用户注册 | 否 |
| `/api/auth/login/` | POST | 登录 | 否 |
| `/api/auth/token/refresh/` | POST | 刷新 Token | 否 |
| `/api/auth/me/` | GET | 当前用户信息 | 是 |
| `/api/auth/me/update/` | PUT | 更新资料 | 是 |
| `/api/auth/creator/become/` | POST | 成为创作者 | 是 |

### 2. 播客 API (`/api/podcasts/`)

#### 节目 (Shows)
| 端点 | 方法 | 说明 | 认证 |
|------|------|------|------|
| `/api/podcasts/shows/` | GET | 节目列表 | 否 |
| `/api/podcasts/shows/create/` | POST | 创建节目 | 是 |
| `/api/podcasts/shows/<slug>/` | GET | 节目详情 | 否 |
| `/api/podcasts/shows/<slug>/update/` | PUT | 更新节目 | 是 |
| `/api/podcasts/shows/<slug>/delete/` | DELETE | 删除节目 | 是 |

**获取节目单集**: `/api/podcasts/episodes/?show_slug=<slug>` (GET, 无需认证)

#### 单集 (Episodes)
| 端点 | 方法 | 说明 | 认证 |
|------|------|------|------|
| `/api/podcasts/episodes/` | GET | 单集列表 | 否 |
| `/api/podcasts/episodes/<id>/` | GET | 单集详情 | 是 |
| `/api/podcasts/episodes/create/` | POST | 创建单集 | 是 |
| `/api/podcasts/episodes/generate/` | POST | AI 生成单集 | 是 |
| `/api/podcasts/episodes/<id>/play/` | POST | 播放计数 | 否 |

#### 热搜 (Trending)
| 端点 | 方法 | 说明 |
|------|------|------|
| `/api/podcasts/trending/sources/` | GET | 数据源列表 |
| `/api/podcasts/trending/<source>/` | GET | 具体热搜数据 |

**支持的数据源**: bilibili, zhihu, weibo, github, v2ex, douyin, 36kr, juejin, sspai, hackernews 等 56 个

#### 创作者
| 端点 | 方法 | 说明 | 认证 |
|------|------|------|------|
| `/api/podcasts/creator/shows/` | GET | 我的节目 | 是 |
| `/api/podcasts/creator/episodes/` | GET | 我的单集 | 是 |
| `/api/podcasts/creator/generation-queue/` | GET | 生成队列 | 是 |

#### 脚本管理 (Scripts)
| 端点 | 方法 | 说明 | 认证 |
|------|------|------|------|
| `/api/podcasts/episodes/<id>/update-script/` | PUT | 更新单集脚本 | 是 |
| `/api/podcasts/script-sessions/` | GET | 脚本会话列表 | 是 |
| `/api/podcasts/script-sessions/` | POST | 创建脚本会话 | 是 |
| `/api/podcasts/script-sessions/<id>/` | GET | 会话详情 | 是 |
| `/api/podcasts/script-sessions/<id>/` | PUT | 更新会话 | 是 |
| `/api/podcasts/script-sessions/<id>/` | DELETE | 删除会话 | 是 |
| `/api/podcasts/script-sessions/<id>/chat/` | POST | AI对话生成脚本 | 是 |
| `/api/podcasts/script-sessions/<id>/upload/` | POST | 上传参考文件 | 是 |
| `/api/podcasts/script-sessions/<id>/finalize/` | POST | 会转为单集 | 是 |

### 3. 搜索 API (`/api/search/`)

| 端点 | 方法 | 说明 |
|------|------|------|
| `/api/search/?q=关键词` | GET | 搜索 |
| `/api/search/quick/?q=关键词` | GET | 快速搜索 |
| `/api/search/popular/` | GET | 热门搜索 |
| `/api/search/suggestions/?q=前缀` | GET | 搜索建议 |

### 4. 互动 API (`/api/interactions/`)

| 端点 | 方法 | 说明 | 认证 |
|------|------|------|------|
| `/api/interactions/episodes/<id>/comments/` | GET | 评论列表 | 否 |
| `/api/interactions/comments/create/` | POST | 发表评论 | 是 |
| `/api/interactions/episodes/<id>/like/` | POST | 点赞/取消 | 是 |
| `/api/interactions/shows/<id>/follow/` | POST | 关注/取消 | 是 |

## 数据结构

### Show (节目)
```json
{
  "id": 34,
  "title": "节目名称",
  "slug": "show-slug",
  "description": "描述",
  "cover_url": "https://...",
  "content_type": "podcast",
  "creator": {...},
  "episodes_count": 10,
  "followers_count": 5,
  "total_plays": 100,
  "visibility": "public"
}
```

### Episode (单集)
```json
{
  "id": 108,
  "title": "单集标题",
  "slug": "episode-slug",
  "description": "描述",
  "audio_url": "https://...",
  "duration": 238,
  "file_size": 5972365,
  "play_count": 10,
  "like_count": 5,
  "status": "published"
}
```

### TrendingItem (热搜项)
```json
{
  "id": "BV1dScmz7ENx",
  "title": "标题",
  "desc": "描述",
  "hot": 4395,
  "url": "https://...",
  "cover": "https://..."
}
```

### ScriptSession (脚本会话)
```json
{
  "id": 1,
  "title": "AI话题讨论",
  "description": "讨论AI最新进展",
  "content_type": "podcast",
  "current_script": "【主持人】大家好...\n【嘉宾】今天我们来聊聊...",
  "script_versions": [
    {"version": 1, "script": "...", "timestamp": "2026-03-14T10:00:00Z"}
  ],
  "uploaded_files": [...],
  "created_at": "2026-03-14T09:00:00Z"
}
```

## 使用示例

### Python

```python
import requests

BASE_URL = "https://mofa.fm/api"

# 1. 登录获取 token
resp = requests.post(f"{BASE_URL}/auth/login/", json={
    "username": "your_username",
    "password": "your_password"
})
token = resp.json()["access"]
headers = {"Authorization": f"Bearer {token}"}

# 2. 获取节目列表
shows = requests.get(f"{BASE_URL}/podcasts/shows/").json()

# 3. 获取 B 站热搜
trending = requests.get(f"{BASE_URL}/podcasts/trending/bilibili/").json()

# 4. 搜索
results = requests.get(f"{BASE_URL}/search/?q=mofa").json()

# 5. 创建节目 (需认证)
new_show = requests.post(
    f"{BASE_URL}/podcasts/shows/create/",
    headers=headers,
    json={"title": "新节目", "description": "...", "content_type": "podcast"}
).json()

# 6. 创建脚本会话并 AI 生成脚本
session = requests.post(
    f"{BASE_URL}/podcasts/script-sessions/",
    headers=headers,
    json={"title": "AI话题", "description": "讨论AI", "content_type": "podcast"}
).json()

# 7. AI 对话生成脚本
chat = requests.post(
    f"{BASE_URL}/podcasts/script-sessions/{session['id']}/chat/",
    headers=headers,
    json={"message": "帮我写一个关于Claude的播客脚本"}
).json()

# 8. 更新单集脚本（支持 Markdown 格式，使用【角色名】标签）
script = """【主持人】大家好，欢迎收听本期节目
【嘉宾】今天我们来聊聊AI的最新进展..."""
requests.put(
    f"{BASE_URL}/podcasts/episodes/123/update-script/",
    headers=headers,
    json={"script": script}
)

# 9. 上传参考文件到脚本会话
with open("article.pdf", "rb") as f:
    requests.post(
        f"{BASE_URL}/podcasts/script-sessions/{session['id']}/upload/",
        headers=headers,
        files={"file": f}
    )

# 10. 将脚本会话转换为播客单集
episode = requests.post(
    f"{BASE_URL}/podcasts/script-sessions/{session['id']}/finalize/",
    headers=headers,
    json={"show_slug": "my-show", "title": "AI播客第1期"}
).json()
```

### cURL

```bash
# 健康检查
curl https://mofa.fm/api/health/

# 获取节目列表
curl https://mofa.fm/api/podcasts/shows/

# 获取知乎热搜
curl https://mofa.fm/api/podcasts/trending/zhihu/

# 登录
curl -X POST https://mofa.fm/api/auth/login/ \
  -H "Content-Type: application/json" \
  -d '{"username":"...","password":"..."}'

# 创建节目 (带认证)
curl -X POST https://mofa.fm/api/podcasts/shows/create/ \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"title":"新节目","content_type":"podcast"}'
```

## 文档

- Swagger UI: `https://mofa.fm/swagger/`
- ReDoc: `https://mofa.fm/redoc/`

## 状态码

| 状态码 | 含义 |
|--------|------|
| 200 | 成功 |
| 201 | 创建成功 |
| 400 | 请求参数错误 |
| 401 | 未认证 |
| 403 | 无权限 |
| 404 | 资源不存在 |
| 500 | 服务器错误 |

## 限制

- 分页: 默认每页 20 条，可通过 `?page=2` 翻页
- 文件上传: 最大 100MB
- 认证 Token: access token 有效期 1 小时，refresh token 有效期 7 天
