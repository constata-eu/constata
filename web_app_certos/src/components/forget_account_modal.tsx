import { Confirm } from 'react-admin';

const ForgetAccountModal = ({open, handleConfirm, handleCancel}) => <Confirm
  isOpen={open}
  title="certos.login.confirm_logout"
  onConfirm={handleConfirm}
  onClose={handleCancel}
  content="certos.login.confirm_logout_text"
/>

export default ForgetAccountModal;
