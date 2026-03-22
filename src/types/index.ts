export interface GelbooruTag {
  text: string
  tagType: string
  count: number
}

export interface GelbooruPostStatistics {
  size: string
  rating: string
  posted: string
  source: string
  score: number
  image: string
}

export interface GelbooruPost {
  id: number
  url: string
  title: string
  tagList: GelbooruTag[]
  statistics: GelbooruPostStatistics
  thumbnail?: string
}

export interface GelbooruPage {
  page: string
  s: string
  tags: string[]
  pageNum: number
  tagList: GelbooruTag[]
  postList: GelbooruPost[]
}

export interface DownloadTask {
  id: number
  postId: number
  imageUrl: string
  fileName: string
  status: 'pending' | 'downloading' | 'completed' | 'failed' | 'paused'
  progress: number
  totalSize: number
  downloadedSize: number
  error?: string
}

export interface AppSettings {
  theme: 'light' | 'dark'
  downloadPath: string
  concurrentDownloads: number
}
