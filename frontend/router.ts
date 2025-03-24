import { createMemoryHistory, createRouter, createWebHistory, type RouteRecordRaw, RouterLink } from 'vue-router'

import LoginView from './components/Login.vue'
import { type CSSProperties, h, type RendererElement, type RendererNode, type VNode } from 'vue'
import {NButton} from 'naive-ui';
import { DocumentsList } from './components/documets_list/documents_list.tsx';
import { RedactionsList } from './components/redactions_list/redactions_list.tsx';
import { user_store } from './store/index.ts';
import { type UserRole } from './models/user.ts';
import user_service from './services/user_service.ts';
import DocumentComparator from './components/document_comparator/DocumentComparator.vue';
export interface IRouteMeta extends Record<string | number | symbol, unknown>
{
  title:  string,
  need_auth: boolean,
  need_role: UserRole
}
export type RouteNames = 'root' | 'documents' | 'redactions' | 'login';
class Route
{
  path: string;
  name: RouteNames;
  redirect?: never;
  meta: IRouteMeta;
  component: VNode<RendererNode, RendererElement, 
  {
    [key: string]: any;
  }>
  constructor(path: string, name: RouteNames, meta: IRouteMeta, component: VNode<RendererNode, RendererElement, {[key: string]: any;}>, redirect?: RouteNames)
    {
      this.path = path;
      this.name = name;
      this.component = component;
      this.meta = meta;
      this.redirect =   { name: redirect } as never;
    }
    get_route(): {path: string, name: string, meta: IRouteMeta, component: VNode<RendererNode, RendererElement, {[key: string]: any;}>}
    {
      return {path: this.path, name: this.name, meta: this.meta, component: this.component}
    }
}

// const routes = [
//   new Route('/', 'root', {title: "Главная страница", need_auth: false}, h("div"), 'login'),
//   new Route('/login', 'login', {title: "Страница входа", need_auth: false}, h(LoginView)),
//   new Route('/documents', 'documents', {title: "Документы", need_auth: true} ,h(DocumentsList)),
//   new Route('/redactions', 'redactions', {title: "Редакции", need_auth: true} ,h(RedactionsList)),
// ]
const routes = [
  {path: '/', name: 'root', meta: {title: "Главная страница", need_auth: false} as IRouteMeta, component: h("div"), redirect: { name: 'login' } as unknown} as RouteRecordRaw,
  {path: '/login', name: 'login', meta: {title: "Страница входа", need_auth: false} as IRouteMeta,  component: h(LoginView)} as RouteRecordRaw,
  {path: '/documents', name: 'documents', meta: {title: "Документы", need_auth: true} as IRouteMeta, component: h(DocumentsList)} as RouteRecordRaw,
  {path: '/redactions', name: 'redactions', meta: {title: "Редакции", need_auth: true} as IRouteMeta , component: h(RedactionsList)} as RouteRecordRaw,
  {path: '/diff',
    query:{id: '', redaction_id: '', doc_hash: '', publication_id: ''},
    props: route => ({id: route.query.id, redaction_id: route.query.redaction_id, doc_hash: route.query.doc_hash, publication_id: route.query.publication_id}),
    name: 'diff',
    meta: {title: "Проверка редакции", need_auth: true} as IRouteMeta,
    component: h(DocumentComparator)} as RouteRecordRaw,
]
export const route_link = (route_name: RouteNames, component: VNode<RendererNode, RendererElement, {[key: string]: any;}>) =>
{
  const button = component_is_visible(route_name) ? h(NButton,
  {
    type: router.currentRoute.value.name == route_name ? 'success' : 'default',
    onClick:()=>
    {
      router.push({name: route_name})
    }
  },
  {default:() => component}): h('span');
  return button;
}

const component_is_visible = (route_name: RouteNames): boolean =>
{
  let visibility = false;
  const vis = router.getRoutes().forEach(f=>
  {
    if (f.name == route_name)
    {
      if ((f.meta as IRouteMeta).need_auth)
      {
        if(user_service.check_token())
        {
          visibility = true;
        }
        else
        {
          visibility = false;
        }
      }
      else
      {
        visibility = true;
      }
    }
  })
  return visibility;
}

const router = createRouter({
  history: createWebHistory(),
  routes,
})

router.afterEach((to, from) => 
{
  const meta = to.meta as IRouteMeta;
  //Vue.nextTick(() => 
  //{
      document.title = meta.title;
  //})
})
router.beforeEach(async (to, from, next) => 
{
  const meta = to.meta as IRouteMeta;
  if (meta.need_auth)
  {
    if(user_service.check_token())
      next()
    else
    {
      if(await user_service.update_tokens())
      {
        next()
      }
      else if(router.currentRoute.value.name != 'login')
        next({name: 'login'})
    } 
  }
  else next()
})
export default router;