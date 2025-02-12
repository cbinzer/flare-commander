import { FunctionComponent } from 'react';
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from '@/components/ui/breadcrumb.tsx';
import { Location, useLocation, useNavigate } from 'react-router';
import { KvNamespace } from '@/kv/kv-models.ts';

const KvNamespaceDetails: FunctionComponent = () => {
  const navigate = useNavigate();
  const location: Location<KvNamespace> = useLocation();
  const namespace = location.state;

  if (!namespace) {
    navigate('/');
    return;
  }

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
              <BreadcrumbPage>{namespace.title}</BreadcrumbPage>
            </BreadcrumbItem>
          </BreadcrumbList>
        </Breadcrumb>
      </header>

      <div className="flex flex-1 flex-col gap-4 p-4">
        <h2 className="scroll-m-20 pb-2 text-3xl font-semibold tracking-tight first:mt-0">
          {namespace.title}
        </h2>
      </div>
    </>
  );
};

export default KvNamespaceDetails;
