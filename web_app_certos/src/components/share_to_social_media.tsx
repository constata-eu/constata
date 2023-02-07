
import { Button } from '@mui/material';
import { useTranslate } from 'react-admin';
import LinkedInIcon from '@mui/icons-material/LinkedIn';
import TwitterIcon from '@mui/icons-material/Twitter';

const ShareToSocialMedia = ({url, icon, id, text}) => {
  const translate = useTranslate();
  let href = url.replaceAll("+", "%2b");
  return <Button
    sx={{mx: 0.5, my: 1}}
    startIcon={icon}
    href={href}
    target="_blank"
    id={id}
    variant="contained"
  >
      { translate(text) }
  </Button>
}

const ShareToLinkedin = ({url, text}) => {
  const href = "https://www.linkedin.com/feed/?shareActive=true&text=" + text + "%20" + url;
  return <ShareToSocialMedia url={href} icon={<LinkedInIcon />} id="share-on-linkedin" text={"certos.download_proof_link.share.linkedin"}/>
}

const ShareToTwitter = ({url, text}) => {
  const href = "https://twitter.com/intent/tweet?url=" + url + "&text=" + text;
  return <ShareToSocialMedia url={href} icon={<TwitterIcon />} id="share-on-twitter" text={"certos.download_proof_link.share.twitter"}/>
}
  
export {ShareToLinkedin, ShareToTwitter};
  
