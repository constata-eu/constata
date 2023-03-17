
import { GraphiQL } from "graphiql";
import { GraphiQLProvider, QueryEditor } from '@graphiql/react';
import { createGraphiQLFetcher } from '@graphiql/toolkit';

import 'graphiql/graphiql.css';

const Graphiql = () => {
  const fetcher = createGraphiQLFetcher({
    url: 'https://localhost:8000/graphql',
   });

   
  let myHeaders = {Authentication: "myheader"};

  return(
    <>
     <div className="graphiql-container">
      HOLAAA
    </div>
      <GraphiQL fetcher={fetcher} headers={myHeaders} />
    </>
  )
}

export default Graphiql;