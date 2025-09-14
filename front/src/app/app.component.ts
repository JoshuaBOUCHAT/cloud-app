import { Component } from '@angular/core';
import { RouterOutlet, provideRouter } from '@angular/router';
import { routes } from './app.routes';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [RouterOutlet], // <-- IMPORTANT !
  templateUrl: './app.component.html',
})
export class AppComponent {}
