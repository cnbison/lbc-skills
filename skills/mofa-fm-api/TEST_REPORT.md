# MoFA FM API 测试报告

**测试时间**: 2026-03-18
**测试目标**: MoFA FM 播客平台 API (https://mofa.fm/api)
**测试范围**: 公开 API + 热搜数据源

---

## 📊 测试总结

| 测试类别 | 通过 | 失败 | 跳过 | 通过率 |
|---------|------|------|------|--------|
| 公开 API | 12 | 0 | 0 | 100% |
| 节目详情 | 2 | 0 | 0 | 100% |
| 单集与评论 | 2 | 0 | 0 | 100% |
| 热搜数据源 | 14 | 0 | 0 | 100% |
| **总计** | **30** | **0** | **0** | **100%** |

**状态**: ✅ 全部通过

---

## ✅ 测试通过的 API

### 1. 系统健康
| 端点 | 方法 | 状态 |
|------|------|------|
| `/health/` | GET | ✅ 正常 |

**响应示例**:
```json
{
  "status": "healthy",
  "services": {
    "database": { "status": "operational" },
    "redis": { "status": "operational" },
    "ai_script": { "status": "operational" },
    "tts": { "status": "operational" }
  }
}
```

### 2. 节目管理
| 端点 | 方法 | 认证 | 状态 |
|------|------|------|------|
| `/podcasts/shows/` | GET | 否 | ✅ 正常 (20个节目) |
| `/podcasts/shows/<slug>/` | GET | 否 | ✅ 正常 |
| `/podcasts/episodes/?show_slug=<slug>` | GET | 否 | ✅ 正常 |

### 3. 单集管理
| 端点 | 方法 | 认证 | 状态 |
|------|------|------|------|
| `/podcasts/episodes/` | GET | 否 | ✅ 正常 (65个单集) |
| `/podcasts/episodes/<id>/play/` | POST | 否 | ✅ 正常 |

### 4. 热搜系统 (56个数据源)
| 端点 | 方法 | 状态 | 数据量 |
|------|------|------|--------|
| `/podcasts/trending/sources/` | GET | ✅ 正常 | 56个源 |
| `/podcasts/trending/bilibili/` | GET | ✅ 正常 | 100条 |
| `/podcasts/trending/zhihu/` | GET | ✅ 正常 | 30条 |
| `/podcasts/trending/weibo/` | GET | ✅ 正常 | 51条 |
| `/podcasts/trending/github/` | GET | ✅ 正常 | 6条 |
| `/podcasts/trending/v2ex/` | GET | ✅ 正常 | 9条 |
| `/podcasts/trending/douyin/` | GET | ✅ 正常 | 49条 |
| `/podcasts/trending/36kr/` | GET | ✅ 正常 | 50条 |
| `/podcasts/trending/juejin/` | GET | ✅ 正常 | 50条 |
| `/podcasts/trending/sspai/` | GET | ✅ 正常 | 40条 |
| `/podcasts/trending/hackernews/` | GET | ✅ 正常 | 30条 |
| `/podcasts/trending/douban-group/` | GET | ✅ 正常 | 30条 |
| `/podcasts/trending/tieba/` | GET | ✅ 正常 | 29条 |
| `/podcasts/trending/csdn/` | GET | ✅ 正常 | 30条 |
| `/podcasts/trending/ithome/` | GET | ✅ 正常 | 48条 |

**支持的热搜源列表**:
```
36kr, 51cto, 52pojie, acfun, baidu, bilibili, coolapk, csdn, dgtle,
douban-group, douban-movie, douyin, earthquake, gameres, geekpark,
genshin, github, guokr, hackernews, hellogithub, history, honkai,
hostloc, hupu, huxiu, ifanr, ithome-xijiayi, ithome, jianshu, juejin,
kuaishou, linuxdo, lol, miyoushe, netease-news, newsmth, ngabbs,
nodeseek, nytimes, producthunt, qq-news, sina-news, sina, smzdm,
sspai, starrail, thepaper, tieba, toutiao, v2ex, weatheralarm, weibo,
weread, yystv, zhihu-daily, zhihu
```

### 5. 搜索系统
| 端点 | 方法 | 状态 |
|------|------|------|
| `/search/?q=<query>` | GET | ✅ 正常 |
| `/search/suggestions/?q=<prefix>` | GET | ✅ 正常 |
| `/search/popular/` | GET | ✅ 正常 (3条热门) |

### 6. 互动系统
| 端点 | 方法 | 状态 |
|------|------|------|
| `/interactions/episodes/<id>/comments/` | GET | ✅ 正常 |

### 7. 统计与推荐
| 端点 | 方法 | 状态 |
|------|------|------|
| `/podcasts/stats/` | GET | ✅ 正常 |
| `/podcasts/recommendations/episodes/` | GET | ✅ 正常 |

### 8. 分类与标签
| 端点 | 方法 | 状态 | 备注 |
|------|------|------|------|
| `/podcasts/categories/` | GET | ✅ 正常 | 当前为空 |
| `/podcasts/tags/` | GET | ✅ 正常 | 当前为空 |

---

## 🔧 发现问题与修复

### 问题 #1: 节目单集端点路径错误
**状态**: ✅ 已修复

**原始代码**:
```python
client._get(f"/podcasts/shows/{slug}/episodes/")  # 返回 404
```

**正确用法**:
```python
client._get("/podcasts/episodes/", params={"show_slug": slug})  # ✅ 正常
```

---

## 📈 平台数据统计

```json
{
  "total_shows": 20,
  "total_episodes": 65,
  "total_creators": 6,
  "total_plays": 74
}
```

---

## 🛠️ CLI 工具测试

所有 CLI 命令测试通过:

```bash
# 系统健康
python tools/fm_client.py health

# 节目列表
python tools/fm_client.py shows

# 单集列表
python tools/fm_client.py episodes

# 热搜数据源
python tools/fm_client.py trending-sources

# 获取热搜
python tools/fm_client.py trending --source bilibili
python tools/fm_client.py trending --source zhihu
python tools/fm_client.py trending --source github

# 搜索
python tools/fm_client.py search --query "AI"

# 统计
python tools/fm_client.py stats
```

---

## 🔐 认证 API (未测试)

以下 API 需要 JWT Token，本次测试未覆盖:

| 端点 | 方法 | 说明 |
|------|------|------|
| `/auth/register/` | POST | 用户注册 |
| `/auth/login/` | POST | 用户登录 |
| `/auth/me/` | GET | 当前用户 |
| `/auth/creator/become/` | POST | 成为创作者 |
| `/podcasts/shows/create/` | POST | 创建节目 |
| `/podcasts/episodes/create/` | POST | 创建单集 |
| `/podcasts/episodes/generate/` | POST | AI生成单集 |
| `/podcasts/creator/shows/` | GET | 我的节目 |
| `/podcasts/creator/episodes/` | GET | 我的单集 |
| `/podcasts/script-sessions/` | GET/POST | 脚本会话 |
| `/interactions/comments/create/` | POST | 发表评论 |
| `/interactions/episodes/<id>/like/` | POST | 点赞 |
| `/interactions/shows/<id>/follow/` | POST | 关注 |

---

## 📁 测试文件

- **测试脚本**: `test_api.py`
- **详细报告**: `test_report.json`
- **客户端代码**: `tools/fm_client.py`

---

## ✅ 结论

MoFA FM API 的公开接口运行正常，所有核心功能（节目浏览、热搜获取、搜索、播放统计等）均可正常使用。热搜系统支持 56 个数据源，数据返回格式统一、完整。

**建议**:
1. 补充认证 API 的自动化测试
2. 添加更多错误场景测试（如无效参数、权限不足等）
3. 考虑添加性能测试（响应时间、并发处理）
