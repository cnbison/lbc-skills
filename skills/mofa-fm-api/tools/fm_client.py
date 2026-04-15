#!/usr/bin/env python3
"""
MoFA FM API Client

封装 MoFA FM 播客平台 API 的 Python 客户端
"""

import requests
import os
from typing import Optional, Dict, Any, List
from urllib.parse import urljoin


class MoFAFMClient:
    """MoFA FM API 客户端"""

    BASE_URL = "https://mofa.fm/api"

    def __init__(self, access_token: Optional[str] = None):
        self.access_token = access_token
        self.session = requests.Session()

    def _headers(self, auth: bool = False) -> Dict[str, str]:
        """构建请求头"""
        headers = {"Content-Type": "application/json"}
        if auth and self.access_token:
            headers["Authorization"] = f"Bearer {self.access_token}"
        return headers

    def _get(self, path: str, auth: bool = False, params: Optional[Dict] = None) -> Dict:
        """GET 请求"""
        url = urljoin(self.BASE_URL + "/", path.lstrip("/"))
        resp = self.session.get(url, headers=self._headers(auth), params=params)
        resp.raise_for_status()
        return resp.json()

    def _post(self, path: str, data: Dict, auth: bool = False) -> Dict:
        """POST 请求"""
        url = urljoin(self.BASE_URL + "/", path.lstrip("/"))
        resp = self.session.post(url, headers=self._headers(auth), json=data)
        resp.raise_for_status()
        return resp.json()

    def _put(self, path: str, data: Dict, auth: bool = False) -> Dict:
        """PUT 请求"""
        url = urljoin(self.BASE_URL + "/", path.lstrip("/"))
        resp = self.session.put(url, headers=self._headers(auth), json=data)
        resp.raise_for_status()
        return resp.json()

    def _patch(self, path: str, data: Dict, auth: bool = False) -> Dict:
        """PATCH 请求"""
        url = urljoin(self.BASE_URL + "/", path.lstrip("/"))
        resp = self.session.patch(url, headers=self._headers(auth), json=data)
        resp.raise_for_status()
        return resp.json()

    def _delete(self, path: str, auth: bool = False) -> Dict:
        """DELETE 请求"""
        url = urljoin(self.BASE_URL + "/", path.lstrip("/"))
        resp = self.session.delete(url, headers=self._headers(auth))
        resp.raise_for_status()
        if resp.status_code != 204:
            return resp.json()
        return {"success": True}

    # ========== 认证 ==========

    def register(self, username: str, email: str, password: str) -> Dict:
        """用户注册"""
        return self._post("/auth/register/", {
            "username": username,
            "email": email,
            "password": password,
            "password2": password
        })

    def login(self, username: str, password: str) -> Dict:
        """用户登录，返回 tokens"""
        result = self._post("/auth/login/", {
            "username": username,
            "password": password
        })
        # 支持两种返回格式: {"access": "..."} 或 {"tokens": {"access": "..."}}
        if "access" in result:
            self.access_token = result["access"]
        elif "tokens" in result and "access" in result["tokens"]:
            self.access_token = result["tokens"]["access"]
        return result

    def me(self) -> Dict:
        """获取当前用户信息"""
        return self._get("/auth/me/", auth=True)

    def become_creator(self) -> Dict:
        """成为创作者"""
        return self._post("/auth/creator/become/", {}, auth=True)

    # ========== 节目 ==========

    def list_shows(self, page: int = 1) -> Dict:
        """获取节目列表"""
        return self._get("/podcasts/shows/", params={"page": page})

    def get_show(self, slug: str) -> Dict:
        """获取节目详情"""
        return self._get(f"/podcasts/shows/{slug}/")

    def create_show(self, title: str, description: str = "", content_type: str = "podcast") -> Dict:
        """创建节目"""
        return self._post("/podcasts/shows/create/", {
            "title": title,
            "description": description,
            "content_type": content_type
        }, auth=True)

    def update_show(self, slug: str, title: str = None, description: str = None, **kwargs) -> Dict:
        """更新节目"""
        data = {}
        if title is not None:
            data["title"] = title
        if description is not None:
            data["description"] = description
        data.update(kwargs)
        return self._put(f"/podcasts/shows/{slug}/update/", data, auth=True)

    def delete_show(self, slug: str) -> Dict:
        """删除节目"""
        return self._delete(f"/podcasts/shows/{slug}/delete/", auth=True)

    # ========== 单集 ==========

    def list_episodes(self, page: int = 1) -> Dict:
        """获取单集列表"""
        return self._get("/podcasts/episodes/", params={"page": page})

    def get_episode(self, episode_id: int) -> Dict:
        """获取单集详情"""
        return self._get(f"/podcasts/episodes/{episode_id}/", auth=True)

    def create_episode(self, show_id: int, title: str, audio_file_path: str = None, **kwargs) -> Dict:
        """
        创建播客单集（支持上传音频文件）

        Args:
            show_id: 节目ID
            title: 单集标题
            audio_file_path: 音频文件路径（可选）
            description: 描述（可选）
            **kwargs: 其他字段如 description, episode_number 等

        Returns:
            创建的单集详情
        """
        url = urljoin(self.BASE_URL + "/", "podcasts/episodes/create/")
        headers = {"Authorization": f"Bearer {self.access_token}"} if self.access_token else {}

        data = {'show_id': show_id, 'title': title, **kwargs}

        if audio_file_path:
            with open(audio_file_path, 'rb') as f:
                files = {'audio_file': (os.path.basename(audio_file_path), f)}
                resp = self.session.post(url, headers=headers, data=data, files=files)
        else:
            resp = self.session.post(url, headers=headers, json=data)
        resp.raise_for_status()
        return resp.json()

    def update_episode(self, episode_id: int, **kwargs) -> Dict:
        """
        更新单集信息

        Args:
            episode_id: 单集ID
            **kwargs: 要更新的字段如 title, description, audio_file 等
        """
        return self._put(f"/podcasts/episodes/{episode_id}/update/", kwargs, auth=True)

    def delete_episode(self, episode_id: int) -> Dict:
        """删除单集"""
        return self._delete(f"/podcasts/episodes/{episode_id}/delete/", auth=True)

    def increment_play_count(self, episode_id: int) -> Dict:
        """增加单集播放计数"""
        return self._post(f"/podcasts/episodes/{episode_id}/play/", {})

    def generate_episode(self, title: str, content: str = "", source_url: str = "", show_slug: str = "",
                         style: str = "conversational", duration: int = 10) -> Dict:
        """
        AI 生成播客单集

        Args:
            title: 单集标题/主题
            content: 内容文本（可选）
            source_url: 参考来源URL（可选）
            show_slug: 目标节目slug（可选）
            style: 风格 (conversational/news/professional)
            duration: 目标时长（分钟）

        Returns:
            生成任务信息
        """
        return self._post("/podcasts/episodes/generate/", {
            "title": title,
            "content": content,
            "source_url": source_url,
            "show_slug": show_slug,
            "generation_config": {
                "style": style,
                "duration": duration
            }
        }, auth=True)

    def generate_episode_from_source(self, title: str, source_url: str, show_slug: str,
                                     description: str = "", content_type: str = "podcast") -> Dict:
        """
        从热搜源生成播客单集

        Args:
            title: 单集标题
            source_url: 热搜源URL
            show_slug: 目标节目slug
            description: 描述
            content_type: 内容类型

        Returns:
            生成任务信息
        """
        return self._post("/podcasts/episodes/generate-from-source/", {
            "title": title,
            "description": description,
            "source_url": source_url,
            "show_slug": show_slug,
            "content_type": content_type
        }, auth=True)

    # ========== 热搜 ==========

    def list_trending_sources(self) -> List[str]:
        """获取热搜数据源列表"""
        result = self._get("/podcasts/trending/sources/")
        return [s["name"] for s in result.get("routes", [])]

    def get_trending(self, source: str) -> Dict:
        """
        获取热搜数据

        常用 source: bilibili, zhihu, weibo, github, v2ex, douyin, 36kr, juejin, sspai
        """
        return self._get(f"/podcasts/trending/{source}/")

    # ========== 搜索 ==========

    def search(self, query: str) -> Dict:
        """搜索"""
        return self._get("/search/", params={"q": query})

    def search_suggestions(self, query: str) -> List[Dict]:
        """搜索建议"""
        result = self._get("/search/suggestions/", params={"q": query})
        return result.get("results", [])

    def popular_searches(self) -> List[Dict]:
        """热门搜索"""
        result = self._get("/search/popular/")
        return result.get("results", [])

    # ========== 互动 ==========

    def list_comments(self, episode_id: int) -> List[Dict]:
        """获取评论列表"""
        result = self._get(f"/interactions/episodes/{episode_id}/comments/")
        return result.get("results", [])

    def create_comment(self, episode_id: int, text: str) -> Dict:
        """发表评论"""
        return self._post("/interactions/comments/create/", {
            "episode": episode_id,
            "text": text
        }, auth=True)

    def like_episode(self, episode_id: int) -> Dict:
        """点赞/取消点赞单集"""
        return self._post(f"/interactions/episodes/{episode_id}/like/", {}, auth=True)

    def follow_show(self, show_id: int) -> Dict:
        """关注/取消关注节目"""
        return self._post(f"/interactions/shows/{show_id}/follow/", {}, auth=True)

    # ========== 分类和标签 ==========

    def list_categories(self) -> List[Dict]:
        """获取分类列表"""
        result = self._get("/podcasts/categories/")
        return result if isinstance(result, list) else result.get("results", [])

    def list_tags(self) -> List[Dict]:
        """获取标签列表"""
        result = self._get("/podcasts/tags/")
        return result if isinstance(result, list) else result.get("results", [])

    # ========== 统计和推荐 ==========

    def get_stats(self) -> Dict:
        """获取平台统计信息"""
        return self._get("/podcasts/stats/")

    def recommended_episodes(self) -> List[Dict]:
        """获取推荐单集"""
        result = self._get("/podcasts/recommendations/episodes/")
        return result if isinstance(result, list) else result.get("results", [])

    # ========== 创作者 ==========

    def my_shows(self) -> List[Dict]:
        """获取我的节目"""
        result = self._get("/podcasts/creator/shows/", auth=True)
        return result if isinstance(result, list) else result.get("results", [])

    def generation_queue(self) -> Dict:
        """获取生成队列"""
        return self._get("/podcasts/creator/generation-queue/", auth=True)

    # ========== 脚本管理 ==========

    def update_episode_script(self, episode_id: int, script: str) -> Dict:
        """
        更新单集脚本

        Args:
            episode_id: 单集 ID
            script: Markdown 格式脚本内容，支持 【角色名】标签

        Returns:
            更新后的单集详情
        """
        return self._patch(f"/podcasts/episodes/{episode_id}/update-script/", {
            "script": script
        }, auth=True)

    # ========== 脚本会话 (ScriptSession) ==========

    def list_script_sessions(self) -> List[Dict]:
        """获取我的脚本会话列表"""
        result = self._get("/podcasts/script-sessions/", auth=True)
        return result if isinstance(result, list) else result.get("results", [])

    def get_script_session(self, session_id: int) -> Dict:
        """获取脚本会话详情"""
        return self._get(f"/podcasts/script-sessions/{session_id}/", auth=True)

    def create_script_session(self, title: str, description: str = "", content_type: str = "podcast") -> Dict:
        """
        创建脚本会话

        Args:
            title: 会话标题
            description: 会话描述
            content_type: 内容类型 (podcast, debate, conference)

        Returns:
            创建的会话详情
        """
        return self._post("/podcasts/script-sessions/", {
            "title": title,
            "description": description,
            "content_type": content_type
        }, auth=True)

    def update_script_session(self, session_id: int, title: str = None, description: str = None) -> Dict:
        """更新脚本会话"""
        data = {}
        if title is not None:
            data["title"] = title
        if description is not None:
            data["description"] = description
        return self._put(f"/podcasts/script-sessions/{session_id}/", data, auth=True)

    def delete_script_session(self, session_id: int) -> Dict:
        """删除脚本会话"""
        return self._delete(f"/podcasts/script-sessions/{session_id}/", auth=True)

    def chat_script_session(self, session_id: int, message: str) -> Dict:
        """
        与 AI 对话生成或修改脚本

        Args:
            session_id: 会话 ID
            message: 用户消息

        Returns:
            AI 响应，包含生成的脚本
        """
        return self._post(f"/podcasts/script-sessions/{session_id}/chat/", {
            "message": message
        }, auth=True)

    def upload_script_reference(self, session_id: int, file_path: str, description: str = "") -> Dict:
        """
        上传参考文件到脚本会话

        Args:
            session_id: 会话 ID
            file_path: 本地文件路径
            description: 文件描述

        Returns:
            上传结果
        """
        url = urljoin(self.BASE_URL + "/", f"podcasts/script-sessions/{session_id}/upload/")
        headers = {"Authorization": f"Bearer {self.access_token}"} if self.access_token else {}

        with open(file_path, 'rb') as f:
            files = {'file': (os.path.basename(file_path), f)}
            data = {'description': description}
            resp = self.session.post(url, headers=headers, data=data, files=files)
            resp.raise_for_status()
            return resp.json()

    def finalize_script_session(self, session_id: int, show_slug: str, title: str = None) -> Dict:
        """
        将脚本会话转换为播客单集

        Args:
            session_id: 脚本会话 ID
            show_slug: 目标节目 slug
            title: 单集标题（可选，默认使用会话标题）

        Returns:
            创建的单集详情
        """
        data = {"show_slug": show_slug}
        if title:
            data["title"] = title
        return self._post(f"/podcasts/script-sessions/{session_id}/finalize/", data, auth=True)


# ========== CLI 接口 ==========

def main():
    import argparse
    import json

    parser = argparse.ArgumentParser(description="MoFA FM API Client")
    parser.add_argument("action", choices=[
        "health", "login", "me", "shows", "episodes", "trending-sources",
        "trending", "search", "search-suggestions", "popular-searches",
        "comments", "create-comment", "my-shows", "generation-queue",
        "update-script", "script-sessions", "create-script-session",
        "chat-script-session", "upload-script-file",
        "create-episode", "update-episode", "delete-episode", "play",
        "create-show", "update-show", "delete-show",
        "like", "follow", "categories", "tags", "stats", "recommended"
    ])
    parser.add_argument("--username", "-u")
    parser.add_argument("--password", "-p")
    parser.add_argument("--token", "-t", help="JWT access token")
    parser.add_argument("--source", "-s", help="热搜数据源")
    parser.add_argument("--query", "-q", help="搜索关键词")
    parser.add_argument("--episode-id", "-e", type=int, help="单集 ID")
    parser.add_argument("--text", help="评论内容")
    parser.add_argument("--show-slug", help="节目 slug")

    # 脚本相关参数
    parser.add_argument("--session-id", type=int, help="脚本会话 ID")
    parser.add_argument("--script", help="脚本内容（文件路径或直接文本）")
    parser.add_argument("--session-title", help="脚本会话标题")
    parser.add_argument("--session-desc", help="脚本会话描述")
    parser.add_argument("--file-path", help="要上传的文件路径")
    parser.add_argument("--content-type", default="podcast", help="内容类型 (podcast/debate/conference)")

    # 节目/单集相关参数
    parser.add_argument("--show-id", type=int, help="节目 ID")
    parser.add_argument("--show-title", help="节目标题")
    parser.add_argument("--show-desc", help="节目描述")
    parser.add_argument("--audio-file", help="音频文件路径")
    parser.add_argument("--episode-title", help="单集标题")
    parser.add_argument("--episode-desc", help="单集描述")
    parser.add_argument("--generation-style", default="conversational", help="AI生成风格")
    parser.add_argument("--duration", type=int, default=10, help="目标时长（分钟）")

    args = parser.parse_args()

    client = MoFAFMClient(access_token=args.token)

    try:
        if args.action == "health":
            result = client._get("/health/")
        elif args.action == "login":
            result = client.login(args.username, args.password)
        elif args.action == "me":
            result = client.me()
        elif args.action == "shows":
            result = client.list_shows()
        elif args.action == "episodes":
            if args.show_slug:
                result = client._get(f"/podcasts/shows/{args.show_slug}/episodes/")
            else:
                result = client.list_episodes()
        elif args.action == "trending-sources":
            sources = client.list_trending_sources()
            result = {"sources": sources, "count": len(sources)}
        elif args.action == "trending":
            result = client.get_trending(args.source)
        elif args.action == "search":
            result = client.search(args.query)
        elif args.action == "search-suggestions":
            result = client.search_suggestions(args.query)
        elif args.action == "popular-searches":
            result = client.popular_searches()
        elif args.action == "comments":
            result = client.list_comments(args.episode_id)
        elif args.action == "create-comment":
            result = client.create_comment(args.episode_id, args.text)
        elif args.action == "my-shows":
            result = client.my_shows()
        elif args.action == "generation-queue":
            result = client.generation_queue()

        # 脚本管理
        elif args.action == "update-script":
            if not args.episode_id or not args.script:
                result = {"error": "需要 --episode-id 和 --script 参数"}
            elif os.path.isfile(args.script):
                with open(args.script, 'r', encoding='utf-8') as f:
                    script_content = f.read()
                result = client.update_episode_script(args.episode_id, script_content)
            else:
                result = client.update_episode_script(args.episode_id, args.script)

        elif args.action == "script-sessions":
            result = client.list_script_sessions()

        elif args.action == "create-script-session":
            if not args.session_title:
                result = {"error": "需要 --session-title 参数"}
            else:
                result = client.create_script_session(
                    args.session_title,
                    args.session_desc or "",
                    args.content_type
                )

        elif args.action == "chat-script-session":
            if not args.session_id or not args.text:
                result = {"error": "需要 --session-id 和 --text 参数"}
            else:
                result = client.chat_script_session(args.session_id, args.text)

        elif args.action == "upload-script-file":
            if not args.session_id or not args.file_path:
                result = {"error": "需要 --session-id 和 --file-path 参数"}
            elif not os.path.isfile(args.file_path):
                result = {"error": f"文件不存在: {args.file_path}"}
            else:
                result = client.upload_script_reference(args.session_id, args.file_path, args.text or "")

        # 节目管理
        elif args.action == "create-show":
            if not args.show_title:
                result = {"error": "需要 --show-title 参数"}
            else:
                result = client.create_show(args.show_title, args.show_desc or "", args.content_type)

        elif args.action == "update-show":
            if not args.show_slug:
                result = {"error": "需要 --show-slug 参数"}
            else:
                data = {}
                if args.show_title:
                    data["title"] = args.show_title
                if args.show_desc:
                    data["description"] = args.show_desc
                result = client.update_show(args.show_slug, **data)

        elif args.action == "delete-show":
            if not args.show_slug:
                result = {"error": "需要 --show-slug 参数"}
            else:
                result = client.delete_show(args.show_slug)

        # 单集管理
        elif args.action == "create-episode":
            if not args.show_id or not args.episode_title:
                result = {"error": "需要 --show-id 和 --episode-title 参数"}
            else:
                result = client.create_episode(
                    args.show_id,
                    args.episode_title,
                    args.audio_file,
                    description=args.episode_desc or ""
                )

        elif args.action == "update-episode":
            if not args.episode_id:
                result = {"error": "需要 --episode-id 参数"}
            else:
                data = {}
                if args.episode_title:
                    data["title"] = args.episode_title
                if args.episode_desc:
                    data["description"] = args.episode_desc
                result = client.update_episode(args.episode_id, **data)

        elif args.action == "delete-episode":
            if not args.episode_id:
                result = {"error": "需要 --episode-id 参数"}
            else:
                result = client.delete_episode(args.episode_id)

        elif args.action == "play":
            if not args.episode_id:
                result = {"error": "需要 --episode-id 参数"}
            else:
                result = client.increment_play_count(args.episode_id)

        # 互动
        elif args.action == "like":
            if not args.episode_id:
                result = {"error": "需要 --episode-id 参数"}
            else:
                result = client.like_episode(args.episode_id)

        elif args.action == "follow":
            if not args.show_id:
                result = {"error": "需要 --show-id 参数"}
            else:
                result = client.follow_show(args.show_id)

        # 分类/标签/统计
        elif args.action == "categories":
            result = client.list_categories()

        elif args.action == "tags":
            result = client.list_tags()

        elif args.action == "stats":
            result = client.get_stats()

        elif args.action == "recommended":
            result = client.recommended_episodes()

        else:
            result = {"error": "Unknown action"}

        print(json.dumps(result, indent=2, ensure_ascii=False))

    except requests.HTTPError as e:
        print(json.dumps({
            "error": "HTTP Error",
            "status_code": e.response.status_code,
            "message": e.response.text
        }, indent=2))
    except Exception as e:
        print(json.dumps({"error": str(e)}, indent=2))


if __name__ == "__main__":
    main()
