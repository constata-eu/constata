import Iframe from 'react-iframe';

const IframeCertificate = ({url, id}) => {
  return <Iframe
    url={url}
    id={id}
    width="100%"
    height="100%"
    position="absolute"
    frameBorder={0}
  />
}

export default IframeCertificate;