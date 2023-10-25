export interface Recents {
    starred: number[];
    other: number[];
}

export type StoryType = 'feature' | 'bug' | 'chore';

export interface Task {
    id: number;
    shortcutId: number | null;
    title: string;
    description: string;
    storyType: StoryType;
    starred: boolean;
}
