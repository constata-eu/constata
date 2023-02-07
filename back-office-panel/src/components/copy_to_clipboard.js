import { Button, useNotify, useTranslate } from 'react-admin';
import { ContentCopy } from '@mui/icons-material';

const ButtonCopy = ({toCopy}) => {
  const notify = useNotify();
  const copyTextToClipboard = () => {
    navigator.clipboard.writeText(toCopy);
    notify("resources.actions.copy_to_clipboard")
  }
  return <Button onClick={copyTextToClipboard}><ContentCopy /></Button>
}

const FieldCopy = ({text}) => <div className='button-copy'>
  <div>{text}</div>
  <ButtonCopy toCopy={text} />
</div>

const FieldCopyWithUrl = ({text, url}) => {
  const translate = useTranslate();
  if (!text) { text = translate('resources.actions.click_here') }

  return(
  <div className='button-copy'>
    <a href={url} target="_blank" rel="noreferrer">{text}</a>
    <ButtonCopy toCopy={url} />  
   </div>
)};


export {FieldCopy, FieldCopyWithUrl};