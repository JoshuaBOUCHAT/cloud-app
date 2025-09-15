import { Routes } from '@angular/router';
import { AuthComponent } from './components/auth/auth.component';
import { VerifyAccountComponent } from './components/verify/verify.component';

export const routes: Routes = [{ path: 'auth', component: AuthComponent }, { path: 'auth/verify', component: VerifyAccountComponent }];
