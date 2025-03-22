import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from '@/components/ui/breadcrumb';
import { FunctionComponent } from 'react';
import { Link, Location, useLocation, useParams } from 'react-router';
import { KvNamespace } from './kv-models';

const KvItemDetails: FunctionComponent = () => {
  const location: Location<KvNamespace> = useLocation();
  const namespace = location.state;
  const { keyName } = useParams();

  return (
    <>
      <header className="flex h-16 shrink-0 items-center gap-2 border-b px-4">
        <Breadcrumb>
          <BreadcrumbList>
            <BreadcrumbItem className="hidden md:block">
              <BreadcrumbPage>KV</BreadcrumbPage>
            </BreadcrumbItem>

            <BreadcrumbSeparator className="hidden md:block" />
            <BreadcrumbItem>
              <BreadcrumbPage>
                <Link to={`/namespaces/${namespace.id}`} state={namespace}>
                  {namespace.title}
                </Link>
              </BreadcrumbPage>
            </BreadcrumbItem>

            <BreadcrumbSeparator className="hidden md:block" />
            <BreadcrumbItem>
              <BreadcrumbPage>
                <BreadcrumbPage>{keyName}</BreadcrumbPage>
              </BreadcrumbPage>
            </BreadcrumbItem>
          </BreadcrumbList>
        </Breadcrumb>
      </header>

      <div className="flex flex-1 flex-col gap-4 p-4">
        <h2 className="scroll-m-20 pb-2 text-3xl font-semibold tracking-tight first:mt-0">Form</h2>
      </div>
    </>
  );
};

export default KvItemDetails;
