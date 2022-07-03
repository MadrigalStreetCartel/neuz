import { atom } from "recoil";

export const activeHelpState = atom<string | null>({
    key: "activeHelpState",
    default: null,
})