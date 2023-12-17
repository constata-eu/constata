import { useState } from 'react';
import { Confirm, useTranslate, useNotify, useRefresh,
         useDataProvider } from 'react-admin';
import { Dialog, Backdrop, DialogTitle, Button, CircularProgress } from '@mui/material';
import { Delete } from '@mui/icons-material';
import { setAccessToken, clearAccessToken } from './auth_provider';


const DeleteDownloadProofLink = ({access_token, setState}) => {
  const dataProvider = useDataProvider();
  const translate = useTranslate();
  const notify = useNotify();
  const refresh = useRefresh();
  const [open, setOpen] = useState(false);
  const [isLoading, setIsLoading] = useState(false);

  const handleClick = () => setOpen(true);
  const handleDialogClose = () => setOpen(false);

  const handleConfirm = async () => {
    setOpen(false);
    setIsLoading(true);
    setAccessToken(access_token);
    try {
      await dataProvider.delete('DownloadProofLink', { id: 1 });
      setState("delete");
    } catch (e) {
      notify("certos.download_proof_link.error_deletion", { type: 'error' });
    }
    clearAccessToken();
    setIsLoading(false);
    refresh();
  }

  return (
    <>
      <Button
        fullWidth sx={{ my: 2}}
        variant="contained"
        color="error"
        startIcon={<Delete />}
        onClick={() => handleClick()}
        id="delete-download-proof-link"
        >
        { translate("certos.download_proof_link.delete.button") }
      </Button>
      <Confirm
        isOpen={open}
        title={translate("certos.download_proof_link.delete.modal_title")}
        content={translate("certos.download_proof_link.delete.modal_content")}
        ConfirmIcon={Delete}
        onConfirm={handleConfirm}
        onClose={handleDialogClose}
      />

      <Backdrop
        sx={{ color: '#fff', zIndex: (theme) => theme.zIndex.drawer + 1 }}
        open={isLoading}
      >
        <Dialog open={isLoading}>
          <DialogTitle><CircularProgress /></DialogTitle>
        </Dialog>
      </Backdrop>
    </>
  );
};

export default DeleteDownloadProofLink;
