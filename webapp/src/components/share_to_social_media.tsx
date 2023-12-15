
import { Button } from '@mui/material';
import { useTranslate } from 'react-admin';
import LinkedInIcon from '@mui/icons-material/LinkedIn';
import ArticleIcon from '@mui/icons-material/Article';

const ShareToSocialMedia = ({url, icon, id, text}) => {
  const translate = useTranslate();
  return <Button
    sx={{mx: 0.5, my: 1}}
    startIcon={icon}
    href={url}
    target="_blank"
    id={id}
    variant="contained"
  >
      { translate(text) }
  </Button>
}

const ShareCertificateInLinkedin = ({entryTitle, linkedinId, expeditionDate, url}) => {
  let href = new URL("https://www.linkedin.com/profile/add?startTask=CERTIFICATION_NAME");
  if (entryTitle) {
    href.searchParams.set("name", entryTitle);
  }
  if (linkedinId) {
    href.searchParams.set("organizationId", linkedinId);
  }
  if (expeditionDate) {
    const date = new Date(expeditionDate);
    href.searchParams.set("issueYear", date.getFullYear().toString());
    href.searchParams.set("issueMonth", (date.getMonth() + 1).toString());
  }
  href.searchParams.set("certUrl", url);
  console.log(href.toString());

  return <ShareToSocialMedia
    url={href.toString()}
    icon={<ArticleIcon />}
    id="share-certificate-in-linkedin"
    text={"certos.download_proof_link.share.add_certificate_to_linkedin"}
  />
}

const ShareToLinkedin = ({url, text}) => {
  const href= new URL("https://www.linkedin.com/feed/?shareActive=true");
  href.searchParams.set("text", text + " " + url);
  return <ShareToSocialMedia url={href.toString()} icon={<LinkedInIcon />} id="share-on-linkedin" text={"certos.download_proof_link.share.linkedin"} />
};

const ShareToTwitter = ({url, text}) => {
  const href= new URL("https://twitter.com/intent/tweet?");
  href.searchParams.set("url", url);
  href.searchParams.set("text", text);
  return <ShareToSocialMedia url={href.toString()} icon={ <>𝕏</> } id="share-on-twitter" text={"certos.download_proof_link.share.twitter"}/>
}
  
export {ShareToLinkedin, ShareToTwitter, ShareCertificateInLinkedin};
  
