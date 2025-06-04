import { invoke } from '@tauri-apps/api/core';

export interface Config {
  cursor_app: string;
  profile_base: string;
  archive_base: string;
  workspace_base: string;
  network_interface: string;
}

export interface SessionInfo {
  name: string;
  path: string;
  created: string;
}

export interface ArchiveInfo {
  name: string;
  path: string;
  created: string;
  original_session: string;
}

export interface CursorSession {
  id: string;
  name: string;
  created_date: string;
  modified_date: string;
  projects: string[];
}

export async function getConfig(): Promise<Config> {
  try {
    return await invoke<Config>('get_config');
  } catch (error) {
    console.error('Error fetching config:', error);
    throw error;
  }
}

export async function updateConfig(config: Config): Promise<Config> {
  try {
    return await invoke<Config>('update_config', { new_config: config });
  } catch (error) {
    console.error('Error updating config:', error);
    throw error;
  }
}

export async function listSessions(): Promise<SessionInfo[]> {
  try {
    return await invoke<SessionInfo[]>('list_sessions');
  } catch (error) {
    console.error('Error listing sessions:', error);
    return [];
  }
}

export async function listArchives(): Promise<ArchiveInfo[]> {
  try {
    return await invoke<ArchiveInfo[]>('list_archives');
  } catch (error) {
    console.error('Error listing archives:', error);
    return [];
  }
}

export async function createSession(name: string): Promise<string> {
  try {
    return await invoke<string>('create_session', { name });
  } catch (error) {
    console.error('Error creating session:', error);
    throw error;
  }
}

export async function launchSession(session: string, spoofMac: boolean = false, connectVpn: boolean = false): Promise<string> {
  try {
    return await invoke<string>('launch_session_cmd', { 
      session, 
      spoof_mac: spoofMac, 
      connect_vpn: connectVpn 
    });
  } catch (error) {
    console.error('Error launching session:', error);
    throw error;
  }
}

export async function spoofMac(interfaceName?: string): Promise<string> {
  try {
    return await invoke<string>('spoof_mac_cmd', { 
      interface: interfaceName 
    });
  } catch (error) {
    console.error('Error spoofing MAC address:', error);
    throw error;
  }
}

export async function archiveSession(session: string): Promise<string> {
  try {
    return await invoke<string>('archive_session', { session });
  } catch (error) {
    console.error('Error archiving session:', error);
    throw error;
  }
}

export async function restoreArchive(archive: string, newSessionName?: string): Promise<string> {
  try {
    return await invoke<string>('restore_archive', { 
      archive, 
      new_session_name: newSessionName 
    });
  } catch (error) {
    console.error('Error restoring archive:', error);
    throw error;
  }
}

export async function deleteSession(session: string): Promise<string> {
  try {
    return await invoke<string>('delete_session', { session });
  } catch (error) {
    console.error('Error deleting session:', error);
    throw error;
  }
}

export async function deleteArchive(archive: string): Promise<string> {
  try {
    return await invoke<string>('delete_archive', { archive });
  } catch (error) {
    console.error('Error deleting archive:', error);
    throw error;
  }
}

export async function randomMac(): Promise<string> {
  try {
    return await invoke<string>('random_mac');
  } catch (error) {
    console.error('Error generating random MAC:', error);
    throw error;
  }
}

export async function getSessions(): Promise<CursorSession[]> {
  try {
    return await invoke<CursorSession[]>('get_sessions');
  } catch (error) {
    console.error('Error fetching sessions:', error);
    return [];
  }
}

export async function saveSession(name: string, projects: string[]): Promise<CursorSession | null> {
  try {
    return await invoke<CursorSession>('save_session', { name, projects });
  } catch (error) {
    console.error('Error saving session:', error);
    return null;
  }
}

export async function deleteSessionById(id: string): Promise<boolean> {
  try {
    await invoke<void>('delete_session', { id });
    return true;
  } catch (error) {
    console.error('Error deleting session:', error);
    return false;
  }
}

export async function openProjects(projects: string[]): Promise<boolean> {
  try {
    await invoke<void>('open_projects', { projects });
    return true;
  } catch (error) {
    console.error('Error opening projects:', error);
    return false;
  }
} 