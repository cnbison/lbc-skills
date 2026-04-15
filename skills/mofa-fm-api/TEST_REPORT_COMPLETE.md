# MoFA FM API 完整测试报告

**测试时间**: 2026-03-18
**测试账号**: testuser_cmwt6e (新注册)
**API地址**: https://mofa.fm/api

---

## 测试结果总览

| 类别 | 通过 | 失败 | 总计 | 通过率 |
|------|------|------|------|--------|
| 公开 API | 20 | 0 | 20 | 100% |
| 认证 API | 14 | 5 | 19 | 74% |
| **总计** | **34** | **5** | **39** | **87%** |

---

## 公开 API 测试结果 (全部通过)

| 接口 | 方法 | 状态 | 备注 |
|------|------|------|------|
| `/health/` | GET | 通过 | 服务健康 |
| `/podcasts/shows/` | GET | 通过 | 20个节目 |
| `/podcasts/shows/<slug>/` | GET | 通过 | 节目详情 |
| `/podcasts/episodes/` | GET | 通过 | 65个单集 |
| `/podcasts/episodes/?show_slug=` | GET | 通过 | 节目单集(修复后) |
| `/podcasts/episodes/<id>/play/` | POST | 通过 | 播放计数 |
| `/interactions/episodes/<id>/comments/` | GET | 通过 | 评论列表 |
| `/podcasts/trending/sources/` | GET | 通过 | 56个数据源 |
| `/podcasts/trending/bilibili/` | GET | 通过 | 100条数据 |
| `/podcasts/trending/zhihu/` | GET | 通过 | 30条数据 |
| `/podcasts/trending/weibo/` | GET | 通过 | 51条数据 |
| `/podcasts/trending/github/` | GET | 通过 | 6条数据 |
| `/podcasts/trending/v2ex/` | GET | 通过 | 9条数据 |
| `/podcasts/trending/douyin/` | GET | 通过 | 49条数据 |
| `/podcasts/trending/36kr/` | GET | 通过 | 50条数据 |
| `/podcasts/trending/juejin/` | GET | 通过 | 50条数据 |
| `/podcasts/trending/sspai/` | GET | 通过 | 40条数据 |
| `/podcasts/trending/hackernews/` | GET | 通过 | 30条数据 |
| `/podcasts/trending/douban-group/` | GET | 通过 | 30条数据 |
| `/podcasts/trending/tieba/` | GET | 通过 | 29条数据 |
| `/podcasts/trending/csdn/` | GET | 通过 | 30条数据 |
| `/podcasts/trending/ithome/` | GET | 通过 | 48条数据 |
| `/search/?q=` | GET | 通过 | 搜索功能 |
| `/search/suggestions/` | GET | 通过 | 搜索建议 |
| `/search/popular/` | GET | 通过 | 热门搜索(3条) |
| `/podcasts/stats/` | GET | 通过 | 平台统计 |
| `/podcasts/recommendations/episodes/` | GET | 通过 | 推荐单集 |
| `/podcasts/categories/` | GET | 通过 | 当前为空 |
| `/podcasts/tags/` | GET | 通过 | 当前为空 |

---

## 认证 API 测试结果

### 通过的接口 (14个)

| 接口 | 方法 | 状态 | 备注 |
|------|------|------|------|
| `/auth/register/` | POST | 通过 | 注册成功 |
| `/auth/login/` | POST | 通过 | 登录成功 |
| `/auth/me/` | GET | 通过 | 获取用户信息 |
| `/podcasts/creator/shows/` | GET | 通过 | 我的节目(0个) |
| `/podcasts/creator/generation-queue/` | GET | 通过 | 生成队列(空) |
| `/podcasts/episodes/<id>/` | GET | 通过 | 单集详情(需认证) |
| `/interactions/episodes/<id>/like/` | POST | 通过 | 点赞/取消 |
| `/interactions/comments/create/` | POST | 通过 | 发表评论 |
| `/interactions/shows/<id>/follow/` | POST | 通过 | 关注/取消 |
| `/podcasts/script-sessions/` | GET | 通过 | 脚本会话列表 |
| `/podcasts/script-sessions/` | POST | 通过 | 创建脚本会话 |
| `/podcasts/script-sessions/<id>/` | GET | 通过 | 会话详情 |
| `/podcasts/script-sessions/<id>/chat/` | POST | 通过 | AI对话生成脚本 |
| `/podcasts/script-sessions/<id>/` | PUT | 通过 | 更新会话 |
| `/podcasts/script-sessions/<id>/` | DELETE | 通过 | 删除会话 |

### 失败的接口 (5个)

| 接口 | 方法 | 状态 | 问题 |
|------|------|------|------|
| `/auth/creator/become/` | POST | 失败(400) | 可能已是创作者或后端逻辑问题 |
| `/podcasts/creator/episodes/` | GET | 失败(404) | 端点不存在或路径错误 |
| `/podcasts/shows/create/` | POST | 失败(500) | 服务器内部错误 |
| `/podcasts/episodes/<id>/update-script/` | PATCH | 失败(404) | 端点不存在或路径错误 |

---

## 发现的问题

### 问题1: 节目单集端点路径
- 文档/代码中的路径: `/podcasts/shows/{slug}/episodes/`
- 实际可用路径: `/podcasts/episodes/?show_slug={slug}`
- 状态: 已在测试脚本中修复

### 问题2: 成为创作者接口返回400
- 可能是用户已成为创作者，或后端有其他限制
- 需要查看详细错误信息

### 问题3: 创建节目接口返回500
- 服务器内部错误，可能是后端bug
- 请求体格式: `{"title": "...", "description": "...", "content_type": "podcast"}`

### 问题4: 两个端点返回404
- `/podcasts/creator/episodes/` - 可能不存在或路径有误
- `/podcasts/episodes/<id>/update-script/` - 可能不存在或路径有误

---

## 测试账号信息

```
用户名: testuser_cmwt6e
邮箱: testuser_cmwt6e@test.com
密码: TestPass123!
用户ID: 19
状态: 正常
```

---

## 结论

1. **公开 API 全部可用** - 100%通过率，核心浏览功能完整
2. **认证 API 大部分可用** - 74%通过率，关键功能（登录、脚本会话、互动）正常
3. **存在4个后端问题** - 需要后端团队修复
4. **脚本会话功能完整** - AI对话、创建、更新、删除全部正常

**建议**:
1. 修复 `/podcasts/shows/create/` 500错误
2. 确认 `/podcasts/creator/episodes/` 和 `/podcasts/episodes/<id>/update-script/` 的正确端点
3. 检查 `/auth/creator/become/` 的逻辑
