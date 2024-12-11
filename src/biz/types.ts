export type SearchType = 'folders' | 'files' | 'both';
export type LevelType='top'|'all';
export interface SearchScenarios{
    name: string
    path: string
    level:LevelType
    target:SearchType
    fileExtensions: string
}
export interface SearchResult {
    name: string;
    path: string;
    is_dir: boolean;
    size: number;
    modified_at:string;
}