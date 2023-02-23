import { Confirm, useDataProvider, useTranslate, useNotify } from 'react-admin';
import { useSafeSetState } from 'ra-core';
import { Button } from '@mui/material';
import ArchiveIcon from '@mui/icons-material/Archive';
import UnarchiveIcon from '@mui/icons-material/Unarchive';


const ArchiveTemplateAction = ({templateId, templateArchived, variant}) => {
  const dataProvider = useDataProvider();
  const translate = useTranslate();
  const notify = useNotify();
  const [confirmArchiveOpen, setConfirmArchiveOpen] = useSafeSetState(false);
  const templateAction = templateArchived ? "unarchive" : "archive";

  const handleArchive = async (action: string) => {
    try {
      const {data} = await dataProvider.update('Template', {
        id: templateId, data: {input: {id: templateId, action}}, previousData: {} 
      });
    } catch (e) {
      notify("resources.Template.errorUpdating", { type: 'error' });
    }
    setConfirmArchiveOpen(false);
  }

  return <>
    <Button
      id=""
      startIcon={templateArchived ? < UnarchiveIcon /> : <ArchiveIcon />}
      variant={variant}
      onClick={() => setConfirmArchiveOpen(true) }
    >
      { templateArchived ?
      translate("resources.Template.unarchive")
      :
      translate("resources.Template.archive")
      }
    </Button>
    <Confirm
      isOpen={confirmArchiveOpen}
      title={templateArchived ? "resources.Template.unarchiveTitle" : "resources.Template.archiveTitle"}
      onConfirm={() => handleArchive(templateAction)}
      onClose={() => setConfirmArchiveOpen(false)}
      content={templateArchived ? "resources.Template.unarchiveContent" : "resources.Template.archiveContent"}
    />
  </>
}

export default ArchiveTemplateAction;
