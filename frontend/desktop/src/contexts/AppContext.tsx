import {
  createContext,
  useContext,
  useMemo,
  useReducer,
  type Dispatch,
  type PropsWithChildren,
  type ReactNode,
} from "react";
import type { ActiveView, AppSettings } from "../types";
import { DEFAULT_SETTINGS } from "../types";

export interface WorkspaceInfo {
  path: string;
  name: string;
  isGitRepo: boolean;
  branch?: string;
}

interface AppState {
  activeView: ActiveView;
  workspace: WorkspaceInfo | null;
  settings: AppSettings;
}

export type AppAction =
  | { type: "SET_VIEW"; view: ActiveView }
  | { type: "SET_WORKSPACE"; workspace: WorkspaceInfo | null }
  | { type: "SET_SETTINGS"; settings: AppSettings };

interface AppContextType extends AppState {
  dispatch: Dispatch<AppAction>;
}

const AppContext = createContext<AppContextType | null>(null);

function appReducer(state: AppState, action: AppAction): AppState {
  switch (action.type) {
    case "SET_VIEW":
      return {
        ...state,
        activeView: action.view,
      };
    case "SET_WORKSPACE":
      return {
        ...state,
        workspace: action.workspace,
      };
    case "SET_SETTINGS":
      return {
        ...state,
        settings: action.settings,
      };
    default:
      return state;
  }
}

const initialAppState: AppState = {
  activeView: "home",
  workspace: null,
  settings: DEFAULT_SETTINGS,
};

export function AppProvider({ children }: PropsWithChildren): ReactNode {
  const [state, dispatch] = useReducer(appReducer, initialAppState);

  const value = useMemo<AppContextType>(
    () => ({
      ...state,
      dispatch,
    }),
    [state],
  );

  return <AppContext.Provider value={value}>{children}</AppContext.Provider>;
}

export function useAppContext(): AppContextType {
  const context = useContext(AppContext);
  if (!context) {
    throw new Error("useAppContext must be used within an AppProvider");
  }
  return context;
}
