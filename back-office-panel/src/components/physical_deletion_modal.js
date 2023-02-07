import { useState } from 'react';
import { Button, Confirm, useTranslate, useNotify, useRefresh,
         useDataProvider } from 'react-admin';
import { Dialog, Backdrop, DialogTitle } from '@mui/material';
import { DeleteForever } from '@mui/icons-material';


const PhysicalDeletionModal = ({orgDeletionId}) => {
  const dataProvider = useDataProvider();
  const translate = useTranslate();
  const notify = useNotify();
  const refresh = useRefresh();
  const [open, setOpen] = useState(false);
  const [isLoading, setIsLoading] = useState(false);

  const handleClick = () => setOpen(true);
  const handleDialogClose = () => setOpen(false);

  const handleConfirm = async (e) => {
    setOpen(false);
    setIsLoading(true);
    try { 
      await dataProvider.create('PhysicalDeletion',
        {data: {org_deletion_id: orgDeletionId}}
      );
    } catch(error) {
      setIsLoading(false);
      notify('resources.errors.default', {type: 'warning'});
      return;
    }
    setIsLoading(false);
    notify("resources.actions.physicalDeletionCompleted");
    refresh();
  }

  return (
    <>
      <Button label="resources.actions.physicalDelete" onClick={handleClick} >
        <DeleteForever/>
      </Button>
      <Confirm
        isOpen={open}
        title={translate("resources.actions.physicalDelete")}
        content={
          <div className="person-physical-delete">
            {translate("resources.actions.physicalDeletionConfirm")}
          </div>}
        ConfirmIcon={DeleteForever}
        onConfirm={handleConfirm}
        onClose={handleDialogClose}
      />

      <Backdrop
        sx={{ color: '#fff', zIndex: (theme) => theme.zIndex.drawer + 1 }}
        open={isLoading}
      >
        <Dialog open={isLoading}>
          <DialogTitle>{translate("resources.actions.loading")}</DialogTitle>
        </Dialog>
      </Backdrop>
    </>
  );
};

export default PhysicalDeletionModal;