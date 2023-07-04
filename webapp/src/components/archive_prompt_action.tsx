import { Confirm, useDataProvider, useTranslate, useNotify, useGetRecordId, useRefresh } from 'react-admin';
import { useSafeSetState } from 'ra-core';
import { Button, IconButton } from '@mui/material';
import DeleteIcon from '@mui/icons-material/Delete';

const ArchivePromptAction = () => {
  const promptId = useGetRecordId();
  const dataProvider = useDataProvider();
  const translate = useTranslate();
  const notify = useNotify();
  const refresh = useRefresh();
  const [confirmArchiveOpen, setConfirmArchiveOpen] = useSafeSetState(false);

  const handleArchive = async () => {
    try {
      await dataProvider.update('VcPrompt', {
        id: promptId, data: {input: {id: promptId, action: "archive"}}, previousData: {} 
      });
    } catch (e) {
      notify("vc_validator.archive_prompt.errorUpdating", { type: 'error' });
    }
    setConfirmArchiveOpen(false);
    refresh();
  }

  return <>
    <IconButton color="error" size="medium" onClick={() => setConfirmArchiveOpen(true) } > <DeleteIcon /> </IconButton>
    <Confirm
      isOpen={confirmArchiveOpen}
      title="vc_validator.archive_prompt.title"
      onConfirm={handleArchive}
      onClose={() => setConfirmArchiveOpen(false)}
      content="vc_validator.archive_prompt.content"
    />
  </>
}

export default ArchivePromptAction;
