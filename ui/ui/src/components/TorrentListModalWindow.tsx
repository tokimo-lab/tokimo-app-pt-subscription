import { TorrentListWindow } from "./TorrentListWindow";

interface WindowHandle {
  id: string;
  metadata: Record<string, unknown>;
  close: () => void;
}

interface Props {
  win: WindowHandle;
}

export default function TorrentListModalWindow({ win }: Props) {
  const clientId = win.metadata.clientId as string;
  const clientName = win.metadata.clientName as string;
  const clientType = (win.metadata.clientType as string) ?? "";

  return (
    <TorrentListWindow
      clientId={clientId}
      clientName={clientName}
      clientType={clientType}
    />
  );
}
