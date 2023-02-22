
import { Button } from '@mui/material';
import { useTranslate } from 'react-admin';
import LinkedInIcon from '@mui/icons-material/LinkedIn';
import TwitterIcon from '@mui/icons-material/Twitter';
import ArticleIcon from '@mui/icons-material/Article';



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

const ShareCertificateInLinkedin = ({entryTitle, linkedinId, expeditionDate, url}) => {
  let origin = "https://www.linkedin.com/profile/add?startTask=CERTIFICATION_NAME";
  if (entryTitle) {
    origin = origin + "&name=" + entryTitle.replaceAll(" ", "%20");
  }
  if (expeditionDate) {
    const date = new Date(expeditionDate);
    origin = origin + "&issueYear=" + date.getFullYear() + "&issueMonth=" + (date.getMonth() + 1);
  }
  if (linkedinId) {
    origin = origin + "&organizationId=" + linkedinId;
  }
  const href = origin + "&certUrl=" + url;
  return <ShareToSocialMedia url={href} icon={<ArticleIcon />} id="share-certificate-in-linkedin" text={"certos.download_proof_link.share.add_certificate_to_linkedin"}/>
}

const ShareToLinkedin = ({url, text}) => {
  const href = "https://www.linkedin.com/feed/?shareActive=true&text=" + text + "%20" + url;
  return <ShareToSocialMedia url={href} icon={<LinkedInIcon />} id="share-on-linkedin" text={"certos.download_proof_link.share.linkedin"} />
};

const ShareToTwitter = ({url, text}) => {
  const href = "https://twitter.com/intent/tweet?url=" + url + "&text=" + text;
  return <ShareToSocialMedia url={href} icon={<TwitterIcon />} id="share-on-twitter" text={"certos.download_proof_link.share.twitter"}/>
}
  
export {ShareToLinkedin, ShareToTwitter, ShareCertificateInLinkedin};
  
